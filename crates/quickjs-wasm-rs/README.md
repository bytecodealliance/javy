# quickjs-wasm-rs

High-level bindings and serializers for a Wasm build of QuickJS.

## Bindings

`Context` corresponds to a QuickJS `JSContext` and `Value` corresponds to a QuickJS `JSValue`.

```rust
use quickjs_wasm_rs::Context;

let mut context = Context::default();
```

will create a new context.

## Serializers

`transcoder` allows importing functions to serialize a json byte array to a `Value` and deserialize from a `Value` to a json byte array.

```rust
use quickjs_wasm_rs::{transcoder, Context, Value};

let context = Context::default();
let input_bytes: &[u8] = ...;
let input_value = transcoder::transcode_input(&context, input_bytes).unwrap();
let output_value: Value = ...;
let output = transcoder::transcode_output(output_value).unwrap();
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
