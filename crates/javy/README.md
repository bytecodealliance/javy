# javy

A configurable JavaScript runtime for WebAssembly.

Uses QuickJS through the `quickjs-wasm-rs` crate to evalulate JavaScript source code or QuickJS bytecode.

## Example usage

```rust
use anyhow::Result;
use javy::Runtime;

fn main() -> Result<()> {
    let runtime = Runtime::default();
    let context = runtime.context();
    context.global_object()?.set_property(
        "print",
        context.wrap_callback(move |ctx, _this, args| {
            let str = args.first().unwrap().to_string();
            println!("{str}");
            Ok(javy::quickjs::from_qjs_value(&ctx.undefined_value()?)?)
        })?,
    )?;
    context.eval_global("hello.js", "print('hello!');")?;
    Ok(())
}
```

Create a `Runtime` and use the reference returned by `context()` to add functions and evaluate source code.

## Building a project using this crate

- Install the [wasi-sdk](https://github.com/WebAssembly/wasi-sdk#install) for your platform
- Set the `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable to the absolute path where you installed the `wasi-sdk`

For example, if you install the `wasi-sdk` in `/opt/wasi-sdk`, you can run:
```bash
export QUICKJS_WASM_SYS_WASI_SDK_PATH=/opt/wasi-sdk
```

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.
