# javy

A configurable JavaScript runtime for WebAssembly.

Uses QuickJS through the [`rquickjs`](https://docs.rs/rquickjs/latest/rquickjs/)
crate to evalulate JavaScript source code or QuickJS bytecode.

## Example usage

```rust
use anyhow::anyhow;
use javy::{Runtime, from_js_error};
let runtime = Runtime::default();
let context = runtime.context();

context.with(|cx| {
    let globals = this.globals();
    globals.set(
        "print_hello",
        Function::new(
            this.clone(),
            MutFn::new(move |_, _| {
                println!("Hello, world!");
            }),
        )?,
    )?;
 });

context.with(|cx| {
    cx.eval_with_options(Default::default(), "print_hello();")
         .map_err(|e| from_js_error(cx.clone(), e))
         .map(|_| ())
});
```

Create a `Runtime` and use the reference returned by `context()` to add functions and evaluate source code.

## Features

- `export_alloc_fns` - exports `canonical_abi_realloc` and `canonical_abi_free` from generated WebAssembly for allocating and freeing memory
- `json` - transcoding functions for converting between `JSValueRef` and JSON
- `messagepack` - transcoding functions for converting between `JSValueRef` and MessagePack

## Publishing to crates.io

To publish this crate to crates.io, run `./publish.sh`.
