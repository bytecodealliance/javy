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
* `console` - Registers an implementation of the `console` API.
* `text_encoding` - Registers implementations of `TextEncoder` and `TextDecoder`.
* `random` - Overrides the implementation of `Math.random` to one that seeds the RNG on first call to `Math.random`. This is helpful to enable when using Wizer to snapshot a Javy Runtime so that the output of `Math.random` relies on the WASI context used at runtime and not the WASI context used when Wizening. Enabling this feature will increase the size of the Wasm module that includes the Javy Runtime and will introduce an additional hostcall invocation when `Math.random` is invoked for the first time.
* `stream_io` - Registers implementations of `Javy.IO.readSync` and `Javy.IO.writeSync`.

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.

## Using a custom WASI SDK

This crate can be compiled using a custom [WASI SDK](https://github.com/WebAssembly/wasi-sdk). When building this crate, set the `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable to the absolute path where you installed the SDK.
