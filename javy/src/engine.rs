#[link(wasm_import_module = "shopify_v1")]
extern "C" {
  pub fn input_len(len: *const usize);
  pub fn input_copy(buffer: *mut u8);
  pub fn output_copy(buffer: *const u8, len: usize);
}

pub fn load() -> Vec<u8> {
  let mut len = 0;
  unsafe { input_len(&mut len); }

  let mut input_buffer = vec![0; len];
  unsafe { input_copy(input_buffer.as_mut_ptr()); }

  input_buffer
}

pub fn store(bytes: &[u8]) {
  unsafe {
    output_copy(bytes.as_ptr(),  bytes.len());
  }
}
