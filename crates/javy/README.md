<div align="center">
  <h1><code>Javy</code></h1>
  <p>
    <strong>A configurable JavaScript runtime for WebAssembly</strong>
  </p>
  <p>
    <a href="https://docs.rs/javy"><img src="https://docs.rs/javy/badge.svg" alt="Documentation Status" /></a>
    <a href="https://crates.io/crates/javy"><img src="https://img.shields.io/crates/v/javy.svg" alt="crates.io status" /></a>
  </p>
</div>


Uses QuickJS through the [`rquickjs`](https://docs.rs/rquickjs/latest/rquickjs/)
crate to evalulate JavaScript source code or QuickJS bytecode.

Refer to the [crate level documentation](https://docs.rs/javy) to learn more.

Example usage:

```rust
use anyhow::Result;
use javy::quickjs::{
   function::{MutFn, Rest},
   Ctx, Function, Value
};
use javy::{from_js_error, Runtime};

fn main() -> Result<()> {
    let runtime = Runtime::default();
    let context = runtime.context();

    context.with(|cx| {
        let globals = cx.globals();
        globals.set(
            "print_hello",
            Function::new(
                cx.clone(),
                MutFn::new(|_: Ctx<'_>, _: Rest<Value<'_>>| {
                    println!("Hello, world!");
                }),
            )?,
        )
    })?;

    context.with(|cx| {
        cx.eval_with_options("print_hello();", Default::default())
            .map_err(|e| from_js_error(cx.clone(), e))
            .map(|_: ()| ())
    })?;

    Ok(())
}
```

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`. You will likely need to
run `git submodule deinit test262` so the working tree is small enough for the
publishing to succeed.
