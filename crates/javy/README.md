# javy

A configurable JavaScript runtime for WebAssembly.

Uses QuickJS through the `quickjs-wasm-rs` crate to evalulate JavaScript source code or QuickJS bytecode.

## Example usage

```rust
use anyhow::{anyhow, Result};
use javy::{quickjs::JSValue, Runtime};

fn main() -> Result<()> {
    let runtime = Runtime::default();
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
    Ok(())
}
```

Create a `Runtime` and use the reference returned by `context()` to add functions and evaluate source code.

## Features
- `json` - transcoding functions for converting between `JSValueRef` and JSON
- `messagepack` - transcoding functions for converting between `JSValueRef` and MessagePack

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.

## Using a custom WASI SDK

This crate can be compiled using a custom [WASI SDK](https://github.com/WebAssembly/wasi-sdk). When building this crate, set the `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable to the absolute path where you installed the SDK.
