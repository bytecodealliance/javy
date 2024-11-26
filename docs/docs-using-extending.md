# Extending

If you want to use Javy for your own project, you may find that the existing
code is not sufficient since you may want to offer custom JavaScript APIs. The
approach we recommend taking is to create your own Javy plugin Wasm module.
This plugin module can then be specified when using the Javy CLI when building
Javy Wasm modules as a `-C plugin` flag when using `javy build`.

To create your own Javy plugin Wasm module, create a new Rust project that
will compile to a library (that is, `cargo init --lib`).

Your `Cargo.toml` should look like:

```toml
[package]
name = "my-plugin-name"
version = "0.1.0"

[lib]
name = "my_plugin_name"
crate-type = ["cdylib"]

[dependencies]
javy-plugin-api = "2.0.0"
```

And `src/lib.rs` should look like:

```rust
use javy_plugin_api::{Config, import_namespace};
use javy_plugin_api::javy::quickjs::prelude::Func;

import_namespace!("my_plugin_name");

#[export_name = "initialize_runtime"]
pub extern "C" fn initialize_runtime() {
    let mut config = Config::default();
    config
        .text_encoding(true)
        .javy_stream_io(true);

    javy_plugin_api::initialize_runtime(config, |runtime| {
        runtime
            .context()
            .with(|ctx| {
                ctx.globals()
                    .set("isThisAPlugin", Func::from(|| "yes it is"))
            })
            .unwrap();
        runtime
    })
    .unwrap();
}
```

You can then run `cargo build --target=wasm32-wasip1 --release` to create a
Wasm module. Then you need to run

```
javy init-plugin <path_to_plugin> -o <path_to_initialized_module>`
```

which will validate and initialize the Javy runtime. This `javy init-plugin`
step is required for the plugin to be useable by the Javy CLI.

See our documentation on [using complex data types in Wasm
functions](./contributing-complex-data-types.md) for how to support Wasm
functions that need to use byte arrays, strings, or structured data.

## The full plugin API

This is the Wasm API the Javy CLI expects Javy plugins to expose. The
`javy-plugin-api` crate will export implementations of all required exported
functions except `initialize_runtime`. `import_namespace!` will define the
`import_namespace` custom section.

### Exported Wasm functions

#### `initialize_runtime() -> ()`

This will initialize a mutable global containing the Javy runtime for use by
`compile_src` and `invoke`.

#### `canonical_abi_realloc(orig_ptr: i32, orig_len: i32, new_ptr: i32, new_len: i32) -> ptr: i32`

This is used to allocate memory in the plugin module.

#### `compile_src(src_ptr: i32, src_len: i32) -> bytecode_wide_ptr: i32`

This is used to compile JavaScript source code to QuickJS bytecode. The return
pointer points to a tuple of `(bytecode_ptr: i32, bytecode_len: i32)` in the
plugin instance's linear memory.

#### `invoke(bytecode_ptr: i32, bytecode_len: i32, fn_name_ptr: i32, fn_name_len: i32) -> ()`

This is used to evaluate the JavaScript code and optionally to call an exported
JS function if `fn_name_ptr` is not `0`.

### Custom sections

#### `import_namespace`

Contains a UTF-8 encoded string. This is used to determine the namespace that
will be used for the Wasm imports in dynamically linked modules built with this
plugin.
