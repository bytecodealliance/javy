use std::path::PathBuf;

fn prebuilt_binary(name: &str, bytes: &[u8]) -> PathBuf {
  which::which(name)
    .unwrap_or_else(|_| {
      let tmp_binary = std::env::temp_dir().join(name);

      if !tmp_binary.exists() {
        std::fs::write(&tmp_binary, bytes)
          .unwrap_or_else(|err| panic!("failed to write to {:?}: {}", &tmp_binary, err));
      }

      tmp_binary
    })
}

pub fn wasm_opt() -> PathBuf {
  let bytes = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/vendor/wasm-opt",
  ));
  prebuilt_binary("wasm-opt", &bytes[..])
}

pub fn wasm_strip() -> PathBuf {
  let bytes = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/vendor/wasm-strip",
  ));
  prebuilt_binary("wasm-strip", &bytes[..])
}


