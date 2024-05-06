# Javy APIs

A collection of APIs for Javy.
 
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
