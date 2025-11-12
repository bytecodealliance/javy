# Extending

If you want to use Javy for your own project, you may find that the existing
code is not sufficient since you may want to offer custom JavaScript APIs. The
approach we recommend taking is to create your own Javy plugin Wasm module.
This plugin module can then be specified when using the Javy CLI when building
Javy Wasm modules as a `-C plugin` flag when using `javy build`.

To create your own Javy plugin Wasm module, create a new Rust project that
will compile to a library (that is, `cargo init --lib`). Javy plugins are
written as WASI preview 1 modules or WASI preview 2 Wasm components but
converted to Wasm modules during the initialization process.

## When to use WASI preview 1 or WASI preview 2

You should write your plugin as a WASI preview 1 plugin if you want to use WASI
preview 1 APIs to interact with modules generated with your plugin. These APIs
include reading from standard input and writing to standard output or standard 
error. The drawback to using WASI preview 1 is support may not continue to be
available in various tools because this is a preview API and maintainers for
some tools and the Rust toolchain may opt to discontinue support.

You should write your plugin as a WASI preview 2 plugin if you can support
using a `wasm32-unknown-unknown` module in your environment. The plugin
initialization process will strip off any WASI preview 2 imports so the
initialized plugin will be a `wasm32-unknown-unknown` module. This means you
will need to create and import your own hostcalls to handle input and output
and cannot rely on WASI preview 1 hostcalls to be available. While using
preview 2 requires more effort since you need to define hostcalls to handle
input and output, it should be more future-proof since you will not be relying
on third parties to continue to provide support for WASI preview 1.

## WASI preview 1 plugins

Your `Cargo.toml` should look like:

```toml
[package]
name = "my-plugin-name"
version = "0.1.0"

[lib]
name = "my_plugin_name"
crate-type = ["cdylib"]

[dependencies]
javy-plugin-api = "5.0.0"
```

And `src/lib.rs` should look like:

```rust
use javy_plugin_api::{import_namespace, javy::quickjs::prelude::Func, Config};

// Set your plugin's import namespace.
import_namespace!("my_plugin_name");

// If you want to import a function from the host, here's how to do it.
#[link(wasm_import_module = "some_other_namespace")]
extern "C" {
    fn imported_function();
}

#[export_name = "initialize-runtime"]
pub extern "C" fn initialize_runtime() {
    let config = Config::default();
    javy_plugin_api::initialize_runtime(config, |runtime| {
        runtime.context().with(|ctx| {
            // Creates a `plugin` variable on the global set to `true`.
            ctx.globals().set("plugin", true).unwrap();
            // Creates an `importedFunc` function on the global which will call
            // the imported function.
            ctx.globals()
                .set("importedFunc", Func::from(|| unsafe { imported_function() }))
                .unwrap();
        });
        runtime
    })
    .unwrap();
}
```

You can then run `cargo build --target=wasm32-wasip1 --release` to create a
Wasm module. Then you need to run

```
javy init-plugin <path_to_plugin> -o <path_to_initialized_module>
```

which will validate and initialize the Javy runtime. This `javy init-plugin`
step is required for the plugin to be useable by the Javy CLI.

See our documentation on [using complex data types in Wasm
functions](./contributing-complex-data-types.md) for how to support Wasm
functions that need to use byte arrays, strings, or structured data.

## WASI preview 2 plugins

Your `Cargo.toml` should look like:

```toml
[package]
name = "my-plugin-name"
version = "0.1.0"

[lib]
name = "my_plugin_name"
crate-type = ["cdylib"]

[dependencies]
javy-plugin-api = "5.0.0"
wit-bindgen = "0.47.0"
```

You'll need a WIT file in `wit/world.wit` that looks like:

```wit
package yournamespace:my-javy-plugin@1.0.0;

world my-javy-plugin {
    export compile-src: func(src: list<u8>) -> result<list<u8>, string>;
    export initialize-runtime: func();
    export invoke: func(bytecode: list<u8>, function: option<string>);
}
```

If you want to use hostcalls in your plugin, you'll also need to include imports in your world. For example, if you wanted to import a function named `imported-function` that takes no arguments and doesn't return anything, it'll look like:

```wit
package yournamespace:my-javy-plugin@1.0.0;

world my-javy-plugin {
    export compile-src: func(src: list<u8>) -> result<list<u8>, string>;
    export initialize-runtime: func();
    export invoke: func(bytecode: list<u8>, function: option<string>);

    import imported-function: func();
}
```

Since Javy's plugin initialization process converts Wasm components to Wasm
modules, parameter and result types will be represented as their lowered core
Wasm equivalents. You can also use `s32`, `f32`, `s64`, and `f64` as the
parameter and result types and use the exported `cabi_realloc` Wasm function to
handle structured data (arrays, strings) as you would have with core Wasm
modules. See our documentation on
[using complex data types in Wasm functions](./contributing-complex-data-types.md)
for more details.

For the world with the imported function, the `src/lib.rs` should look like:

```rust
use javy_plugin_api::{
    javy::{quickjs::prelude::Func, Runtime},
    javy_plugin, Config,
};

wit_bindgen::generate!({ world: "my-javy-plugin", generate_all });

fn config() -> Config {
    Config::default()
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime.context().with(|ctx| {
        // Creates a `plugin` variable on the global set to `true`.
        ctx.globals().set("plugin", true).unwrap();
        // Creates an `importedFunc` function on the global which will call
        // the imported function.
        ctx.globals()
            .set(
                "func",
                Func::from(|| {
                    crate::imported_function();
                }),
            )
            .unwrap();
    });
    runtime
}

struct Component;

// Set your plugin's import namespace.
javy_plugin!("my-javy-plugin", Component, config, modify_runtime);

export!(Component);
```

If you do not want to use the `javy_plugin!` macro for whatever reason, you can use the underlying APIs in `src/lib.rs` directly:

```rust
use std::process;

use javy_plugin_api::javy::Runtime;
use javy_plugin_api::{import_namespace, Config};

wit_bindgen::generate!({ world: "my-javy-plugin", generate_all });

// Set your plugin's import namespace.
import_namespace!("my-javy-plugin");

struct Component;

impl Guest for Component {
    fn invoke(bytecode: Vec<u8>, function: Option<String>) {
        javy_plugin_api::invoke(&bytecode, function.as_deref()).unwrap_or_else(|e| {
            eprintln!("{e}");
            process::abort();
        })
    }

    fn compile_src(src: Vec<u8>) -> Result<Vec<u8>, String> {
        javy_plugin_api::compile_src(&src).map_err(|e| e.to_string())
    }

    fn initialize_runtime() {
        javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap()
    }
}

export!(Component);

fn config() -> Config {
    Config::default()
    
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime
}
```

You can then run `cargo build --target=wasm32-wasip2 --release` to create a
Wasm module. Then you need to run

```
javy init-plugin <path_to_plugin> -o <path_to_initialized_module>
```

which will validate and initialize the Javy runtime. This `javy init-plugin`
step is required for the plugin to be useable by the Javy CLI.

## Migration to v2.0.0 of javy-plugin-api

Consult the `javy-plugin-api` README.

## The full plugin API

This is the Wasm API the Javy CLI expects Javy plugins to expose. The
`javy-plugin!` macro defines default implementations of these functions.

### Exported Wasm functions

#### `initialize_runtime() -> ()`

This will initialize a mutable global containing the Javy runtime for use by
`compile_src` and `invoke`.

#### `cabi_realloc(orig_ptr: i32, orig_len: i32, new_ptr: i32, new_len: i32) -> ptr: i32`

This is used to allocate memory in the plugin module.

#### `compile_src(src_ptr: i32, src_len: i32) -> result_wide_ptr: i32`

This is used to compile JavaScript source code to QuickJS bytecode. The return
pointer points to a result type of `(discriminator: i32, ptr: i32, len: i32)` in
the plugin instance's linear memory. If `discriminator` is `0`, `ptr` and `len`
are the offset and length of the QuickJS bytecode. If the `discriminator` is
`1`, `ptr` and `len` are the offset and length of a UTF-8 string containing an
error message.

#### `invoke(bytecode_ptr: i32, bytecode_len: i32, fn_name_discriminator: i32, fn_name_ptr: i32, fn_name_len: i32) -> ()`

This is used to evaluate the JavaScript code and optionally to call an exported
JS function if `fn_name_discriminator` is not `0`.

### Custom sections

#### `import_namespace`

Contains a UTF-8 encoded string. This is used to determine the namespace that
will be used for the Wasm imports in dynamically linked modules built with this
plugin.
