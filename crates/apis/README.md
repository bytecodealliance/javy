# Javy APIs

A collection of APIs for Javy.
 
Example usage:

```rust
use anyhow::{anyhow, Error, Result};
use javy::{quickjs::JSValue, Runtime};
use javy_apis::RuntimeExt;

let runtime = Runtime::new_with_defaults()?;
let context = runtime.context();
context.global_object()?.set_property(
    "print",
    context.wrap_callback(move |_ctx, _this, args| {
        let str = args
            .first()
            .ok_or(anyhow!("Need to pass an argument"))?
            .to_string();
        println!("{str}");
       Ok(JSValue::Undefined)
    })?,
 )?;
 context.eval_global("hello.js", "print('hello!');")?;
```

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.
