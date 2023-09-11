# Using complex data types in Wasm functions

Core WebAssembly currently only supports using numbers for arguments and return values for exported and imported functions. This presents a problem when you want to pass strings, byte arrays, or structured data to and from imported/exported functions. The WebAssembly Component Model provides one approach to solving this problem but we have not yet added support for producing WebAssembly components to Javy. This document will provide an overview for an approach using Core WebAssembly to consider.

At a high level, byte arrays can be passed using a pair of integers with the first integer representing the address of the start of the byte array in the instance's linear memory and the second integer representing the length of the byte array. Strings can be passed by encoding the string into a UTF-8 byte array and using the previous solution to pass the byte array. Structured data can be encoded to a JSON string and that string can be passed by encoding it into a UTF-8 byte array and using the previous solution. Other serialization formats can also be used to encode the structured data to a byte array.

The examples below use Rust and Wasmtime to on the host however any programming language and WebAssembly runtime should support using the same approach.

## For exported functions

Given an exported function that receives a byte array and looks like:

```wat
(module
  (func (export "your_fn") (param $ptr i32) (param $len i32)
    ...)
  (func (export "canonical_abi_realloc") (param $orig_ptr i32) (param $orig_size i32) (param $alignment i32) (param $new_len i32)
    ...)
)
```

Then you can pass a byte array to that exported function from the WebAssembly host:

```rust
use anyhow::Result;

fn call_the_export(bytes: &[u8], instance: wasmtime::Instance, store: &mut wasmtime::Store<WasiCtx>) -> Result<()> {
    let memory = instance.get_memory(&mut store, "memory");
    let realloc_fn = instance
        .get_typed_func::<(u32, u32, u32, u32), u32>(&mut store, "canonical_abi_realloc")?;
    let len = bytes.len().try_into()?;

    let original_ptr = 0;
    let original_size = 0;
    let alignment = 1;
    let ptr =
        realloc_fn.call(&mut store, (original_ptr, original_size, alignment, len))?;

    memory.write(&mut store, ptr.try_into()?, bytes)?;

    let your_fn = instance.get_typed_func::<(u32, u32), ()>(&mut store, "your_fn")?;
    your_fn.call(&mut store, (ptr, len))?;

    Ok(())
}
```

You can export the `canonical_abi_realloc` function by enabling the `export_alloc_fns` feature in the `javy` crate.

In the WebAssembly instance when receiving a byte array in the exported function, you can use the `std::slice::from_raw_parts` function to get the slice.

```rust
#[export_name = "your_fn"]
pub unsafe extern "C" fn your_fn(ptr: *const u8, len: usize) {
    let bytes = std::slice::from_raw_parts(ptr, len);
    todo!(); // use `bytes` for something
}
```

Given an exported WebAssembly function that returns a byte array and looks like:

```wat
(module
  (func (export "your_fn") (result i32)
    ...)
)
```

To return a byte array from that exported function in a WebAssembly instance, you need to leak the byte array and we recommend using a static wide pointer for storing the pointer and length.

```rust
static mut BYTES_RET_AREA: [u32; 2] = [0; 2];

#[export_name = "your_fn"]
pub unsafe extern "C" fn your_fn() -> *const u32 {
    let bytes = todo!(); // fill in your own logic
    let len = bytes.len();
    let ptr = Box::leak(bytes.into_boxed_slice()).as_ptr();
    BYTES_RET_AREA[0] = ptr as u32;
    BYTES_RET_AREA[1] = len.try_into().unwrap();
    BYTES_RET_AREA.as_ptr()
}
```

On the host, you can use `memory.read` to populate a vector with the byte array. WebAssembly uses little-endian integers so we read 32-bit integers using `from_le_bytes`.

```rust
fn get_slice(instance: wasmtime::Instance, store: &mut wasmtime::Store) -> Result<Vec<u8>> {
    let your_fn = instance.get_typed_func::<(), u32>(&mut store, "your_fn")?;
    let ret_ptr = your_fn.call(&mut store, (ptr, len))?;

    let memory = instance.get_memory(&mut store, "memory")?;

    let mut ret_buffer = [0; 8];
    memory.read(&mut store, ret_ptr.try_into()?, &mut ret_buffer)?;

    let bytecode_ptr = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
    let bytecode_len = u32::from_le_bytes(ret_buffer[4..8].try_into()?);

    let mut bytecode = vec![0; bytecode_len.try_into()?];
    memory.read(&mut store, bytecode_ptr.try_into()?, &mut bytecode)?;

    Ok(bytecode)
}
```

## For imported functions

Given an imported WebAssembly function that receives a byte array as an argument and looks like:

```wat
(module
  (import "host" "my_import" (func $my_import (param i32) (param i32)))
)
```

When passing a byte array to the host from the WebAssembly instance, we pass the pointer and length to the imported function:

```rust
use anyhow::Result;

#[link(name = "host")]
extern "C" {
    fn my_import(ptr: *const u32, len: u32);
}

fn call_the_import(bytes: &[u8]) -> Result<()> {
    unsafe { my_import(bytes.as_ptr(), bytes.len().try_into()?) };
}
```

When receiving a byte array from the WebAssembly instance on the host, we use `memory.read` along with the pointer and length to get the byte array:

```rust
use anyhow::Result;

struct StoreContext {
    bytes: Vec<u8>,
    wasi: wasmtime_wasi::WasiCtx,
}

fn setup(linker: &mut wasmtime::Linker<StoreContext>) -> Result<()> {
    wasmtime_wasi::sync::add_to_linker(&mut linker, |ctx: &mut StoreContext| &mut ctx.wasi)?;

    linker
        .func_wrap(
            "host",
            "my_import",
            |mut caller: wasmtime::Caller<'_, StoreContext>, ptr: u32, len: u32| {
                let mut bytes = Vec::with_capacity(len.try_into()?);
                caller
                    .get_export("memory")?
                    .into_memory()
                    .unwrap()
                    .read(&caller, ptr.try_into().unwrap(), &mut bytes)?;
                caller.data_mut().bytes = bytes;
            },
        )?;
}
```

Given an imported WebAssembly function that returns a byte array and looks like:

```wat
(module
  (import "host" "my_import" (func $my_import (result i32)))
  (func (export "canonical_abi_realloc") (param $orig_ptr i32) (param $orig_size i32) (param $alignment i32) (param $new_len i32)
    ...)
)
```

When returning a byte array from the host, things get a little more complicated. Below we use a wide pointer to return the byte array. This requires two memory allocations in the instance, one for the byte array and one for the wide pointer, and using `memory.write` to place the array and wide pointer into the allocated memory. Since the byte array is copied into the instance's memory, there is no need to leak the original byte array.

```rust
fn setup(linker: &mut wasmtime::Linker<wasmtime_wasi::WasiContext>) -> Result<()> {
    wasmtime_wasi::sync::add_to_linker(&mut linker, |ctx: &mut wasmtime_wasi::WasiContext| &mut ctx)?;

    linker
        .func_wrap(
            "host",
            "my_import",
            |mut caller: wasmtime::Caller<'_, StoreContext>| -> Result<u32> {
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let realloc = caller
                    .get_export("canonical_abi_realloc")
                    .unwrap()
                    .into_func()
                    .unwrap()
                    .typed::<(u32, u32, u32, u32), u32>(&caller)?;

                let bytes = todo!();
                let original_ptr = 0;
                let original_size = 0;
                let alignment = 1;
                let ptr = realloc.call(
                    &mut caller,
                    (
                        original_ptr,
                        original_size,
                        alignment,
                        bytes.len().try_into()?,
                    ),
                )?;

                memory.write(&mut caller, ptr.try_into().unwrap(), &bytes)?;

                const LEN: usize = 8;
                let mut wide_ptr_buffer = [0u8; LEN];
                wide_ptr_buffer[0..4].copy_from_slice(&ptr.to_le_bytes());
                wide_ptr_buffer[4..8]
                    .copy_from_slice(&TryInto::<u32>::try_into(bytes.len())?.to_le_bytes());
                let wide_ptr = realloc.call(
                    &mut caller,
                    (original_ptr, original_size, alignment, LEN.try_into()?),
                )?;
                memory.write(&mut caller, wide_ptr.try_into()?, &wide_ptr_buffer)?;
                Ok(wide_ptr)
            },
        )
        .unwrap();
}
```

You can export the `canonical_abi_realloc` function by enabling the `export_alloc_fns` feature in the `javy` crate.

When reading a returned byte array from the host, we extract the pointer and length from the wide pointer and then use the pointer and length to read a slice from memory:

```rust
#[link(wasm_import_module = "host")]
extern "C" {
    fn my_import() -> *const u32;
}

fn main() {
    let bytes = unsafe {
        let wide_ptr = my_import();
        let [ptr, len] = std::slice::from_raw_parts(wide_ptr, 2) else {
            unreachable!()
        };
        std::slice::from_raw_parts(*ptr as *const u8, (*len).try_into().unwrap())
    };
    todo!();
}
```
