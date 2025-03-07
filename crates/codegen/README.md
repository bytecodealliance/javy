<div align="center">
  <h1><code>javy-codegen</code></h1>
  <p>
    <strong>A crate for generating Wasm modules using Javy</strong>
  </p>
  <p>
    <a href="https://docs.rs/javy-codegen"><img src="https://docs.rs/javy-codegen/badge.svg" alt="Documentation Status" /></a>
    <a href="https://crates.io/crates/javy-codegen"><img src="https://img.shields.io/crates/v/javy-codegen" alt="crates.io status" /></a>
  </p>
</div>

Refer to the [crate level documentation](https://docs.rs/javy-codegen) to learn more.

Example usage:

```rust
use std::path::Path;
use javy_codegen::{Generator, LinkingKind, Plugin, JS};

fn main() {
  // Load your target Javascript.
  let js = JS::from_file(Path::new("example.js"));

  // Load existing pre-initialized Javy plugin.
  let plugin = Plugin::new_from_path(Path::new("example-plugin.wasm"));

  // Configure code generator.
  let mut generator = Generator::new();
  generator.plugin(plugin);
  generator.linking(LinkingKind::Static);

  // Generate your Wasm module.
  let wasm = generator.generate(&js)?;
}
```
