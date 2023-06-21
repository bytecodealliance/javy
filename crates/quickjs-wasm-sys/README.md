# quickjs-wasm-sys: Wasm QuickJS bindings for Rust

FFI bindings for a Wasm build of the QuickJS Javascript engine.

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.

## Using a custom WASI SDK

This crate can be compiled using a custom [WASI SDK](https://github.com/WebAssembly/wasi-sdk). When building this crate, set the `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable to the absolute path where you installed the SDK.
