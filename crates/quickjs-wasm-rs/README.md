# quickjs-wasm-rs

High-level bindings and serializers for a Wasm build of QuickJS.

## Bindings

`JSContextRef` corresponds to a QuickJS `JSContext` and `JSValueRef` corresponds to a QuickJS `JSValue`.

```rust
use quickjs_wasm_rs::JSContextRef;

let mut context = JSContextRef::default();
```

will create a new context.

## Serializers

This crate provides optional transcoding features for converting between
serialization formats and `JSValueRef`:
- `messagepack` provides `quickjs_wasm_rs::messagepack` for msgpack, using `rmp_serde`.
- `json` provides `quickjs_wasm_rs::json` for JSON, using `serde_json`.

msgpack example:

```rust
use quickjs_wasm_rs::{messagepack, JSContextRef, JSValueRef};

let context = JSContextRef::default();
let input_bytes: &[u8] = ...;
let input_value = messagepack::transcode_input(&context, input_bytes).unwrap();
let output_value: JSValueRef = ...;
let output = messagepack::transcode_output(output_value).unwrap();
```

## Building a project using this crate

- Install the [wasi-sdk](https://github.com/WebAssembly/wasi-sdk#install) for your platform
- Set the `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable to the absolute path where you installed the `wasi-sdk`

For example, if you install the `wasi-sdk` in `/opt/wasi-sdk`, you can run:
```bash
export QUICKJS_WASM_SYS_WASI_SDK_PATH=/opt/wasi-sdk
```

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.
