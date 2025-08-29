<div align="center">
  <h1><code>javy-plugin-api</code></h1>
  <p>
    <strong>A crate for creating Javy plugins</strong>
  </p>
  <p>
    <a href="https://docs.rs/javy-plugin-api"><img src="https://docs.rs/javy-plugin-api/badge.svg" alt="Documentation Status" /></a>
    <a href="https://crates.io/crates/javy-plugin-api"><img src="https://img.shields.io/crates/v/javy-plugin-api.svg" alt="crates.io status" /></a>
  </p>
</div>

Refer to the [crate level documentation](https://docs.rs/javy-plugin-api) to learn more.

Example usage:

```rust
use javy_plugin_api::{
    javy::{quickjs::prelude::Func, Runtime},
    javy_plugin, Config,
};

wit_bindgen::generate!({ world: "my-javy-plugin-v1", generate_all });

fn config() -> Config {
    let mut config = Config::default();
    config
        .text_encoding(true)
        .javy_stream_io(true);
    config
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime.context().with(|ctx| {
        ctx.globals().set("plugin", true).unwrap();
    });
    runtime
}

struct Component;

// Dynamically linked modules will use `my_javy_plugin_v1` as the import
// namespace.
javy_plugin!("my-javy-plugin-v1", Component, config, modify_runtime);

export!(Component);
```

## Migration for v2.0.0

Please read the extending Javy documentation for the new plugin API details.

To update your plugin:
1. Create a directory named `wit` adjacent to the `Cargo.toml` file
2. Inside that `wit` directory, create a WIT file matching the details in the
   extending Javy documentation
3. Create a `config() -> Config` function that will return a Javy config
4. Create a `modify_runtime(runtime: Runtime) -> Runtime` function that will
   perform whatever modifications to `runtime` that are necessary
5. Add a dependency on `wit-bindgen` to your crate
6. Use `wit_bindgen::generate!` to create the required traits from the WIT file
7. Create a struct type
8. Remove `import_namespace!` and use `javy_plugin!` instead. Ensure you use a
   different value for the `import_namespace` parameter since the API has
   changed. Pass your struct type, your `config` function, and your
   `modify_runtime` function.
9. Call `export!` on your struct type.

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.
