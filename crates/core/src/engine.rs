#[cfg(feature = "standalone-wasi")]
use std::io::{copy, stdin, stdout};

#[cfg(not(feature = "standalone-wasi"))]
#[link(wasm_import_module = "shopify_v1")]
extern "C" {
    pub fn input_len(len: *const usize) -> u32;
    pub fn input_copy(buffer: *mut u8) -> u32;
    pub fn output_copy(buffer: *const u8, len: usize) -> u32;
}

pub fn load() -> Vec<u8> {
    #[cfg(not(feature = "standalone-wasi"))]
    return load_from_abi();

    #[cfg(feature = "standalone-wasi")]
    return load_from_stdin();
}

#[cfg(feature = "standalone-wasi")]
fn load_from_stdin() -> Vec<u8> {
    let mut reader = stdin();
    let mut writer: Vec<u8> = vec![];
    copy(&mut reader, &mut writer).expect("Couldn't read from stdin");

    writer.clone()
}

#[cfg(not(feature = "standalone-wasi"))]
fn load_from_abi() -> Vec<u8> {
    let len = 0;
    unsafe {
        input_len(&len);
    }

    let mut input_buffer = vec![0; len];
    unsafe {
        input_copy(input_buffer.as_mut_ptr());
    }

    input_buffer
}

pub fn store(bytes: &mut [u8]) {
    #[cfg(not(feature = "standalone-wasi"))]
    unsafe {
        store_to_abi(&bytes)
    };

    #[cfg(feature = "standalone-wasi")]
    store_to_stdout(&mut bytes.as_ref());
}

#[cfg(not(feature = "standalone-wasi"))]
unsafe fn store_to_abi(bytes: &[u8]) {
    output_copy(bytes.as_ptr(), bytes.len());
}

#[cfg(feature = "standalone-wasi")]
fn store_to_stdout(bytes: &mut &[u8]) {
    copy(bytes, &mut stdout()).expect("Couldn't copy to stdout");
}
