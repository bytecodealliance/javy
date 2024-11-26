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
use javy_plugin_api::import_namespace;
use javy_plugin_api::Config;

// Dynamically linked modules will use `my_javy_plugin_v1` as the import
// namespace.
import_namespace!("my_javy_plugin_v1");

#[export_name = "initialize_runtime"]
pub extern "C" fn initialize_runtime() {
    let mut config = Config::default();
    config
        .text_encoding(true)
        .javy_stream_io(true);

    javy_plugin_api::initialize_runtime(config, |runtime| runtime).unwrap();
}
```

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.
