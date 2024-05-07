<div align="center">
  <h1><code>Javy APIs</code></h1>
  <p>
    <strong>A collection of APIs for Javy</strong>
  </p>

  <p>
    <a href="https://docs.rs/javy-apis"><img src="https://docs.rs/javy-apis/badge.svg" alt="Documentation Status" /></a>
    <a href="https://crates.io/crates/javy-apis"><img src="https://img.shields.io/crates/v/javy-apis.svg" alt="crates.io status" /></a>
  </p>
</div>

Refer to the [crate level documentation](https://docs.rs/javy-apis) to learn more.
 
Example usage:

```rust
// With the `console` feature enabled.
use javy::{Runtime, from_js_error};
use javy_apis::RuntimeExt;
use anyhow::Result;

fn main() -> Result<()> {
    let runtime = Runtime::new_with_defaults()?;
    let context = runtime.context();
    context.with(|cx| {
        cx.eval_with_options(Default::default(), "console.log('hello!');")
            .map_err(|e| to_js_error(cx.clone(), e))?
    });
    Ok(())
}
```

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.
