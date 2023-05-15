# javy-apis

A collection of APIs that can be added to a Javy runtime.

APIs are registered by enabling crate features.

## Example usage

```rust
use javy::{quickjs::JSValue, Runtime};
// with `console` feature enabled
use javy_apis::RuntimeExt;

fn main() -> Result<()> {
    let runtime = Runtime::new_with_defaults()?;
    let context = runtime.context();
    context.eval_global("hello.js", "console.log('hello!');")?;
    Ok(())
}
```

If you want to customize the runtime or the APIs, you can use the `Runtime::new_with_apis` method instead to provide a `javy::Config` for the underlying `Runtime` or an `APIConfig` for the APIs.

## Features
* `console` - registers an implementation of the `console` API
* `text_encoding` - registers implementations of `TextEncoder` and `TextDecoder`
* `stream_io` - registers implementations of `Javy.IO.readSync` and `Javy.IO.writeSync`

## Building a project using this crate

- Install the [wasi-sdk](https://github.com/WebAssembly/wasi-sdk#install) for your platform
- Set the `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable to the absolute path where you installed the `wasi-sdk`

For example, if you install the `wasi-sdk` in `/opt/wasi-sdk`, you can run:
```bash
export QUICKJS_WASM_SYS_WASI_SDK_PATH=/opt/wasi-sdk
```

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.
