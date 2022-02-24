# quickjs-wasm-rs

Bindings and serializers for a Wasm build of QuickJS.

## Bindings

`Context` corresponds to a QuickJS `JSContext` and `Value` corresponds to a QuickJS `JSValue`.

```rust
use quickjs_wasm_rs::Context;

let mut context = Context::default();
```

will create a new context.

## Serializers

Enabling the `messagepack` feature allows importing functions to serialize a messagepack byte array to a `Value` and deserialize from a `Value` to a messagepack byte array.

```rust
use quickjs_wasm_rs::{messagepack, Context, Value};

let context = Context::default();
let input_bytes: &[u8] = ...;
let input_value = messagepack::transcode_input(&context, input_bytes).unwrap();
let output_value: Value = ...;
let output = messagepack::transcode_output(output_value).unwrap();
```
