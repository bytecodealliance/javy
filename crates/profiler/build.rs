use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let workspace_root = PathBuf::from(&manifest_dir);
    let workspace_root = workspace_root
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let lib_path = workspace_root.join("target/wasm32-wasip1/release/javy_profiler_lib.wasm");

    println!("cargo:rerun-if-changed={}", lib_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let dst = Path::new(&out_dir).join("profiler_lib.wasm");

    if lib_path.exists() {
        fs::copy(&lib_path, &dst).expect("copy profiler_lib.wasm into OUT_DIR");
        return;
    }

    // Stub the profiler lib for clippy.
    if is_clippy() {
        fs::write(&dst, []).expect("write profiler_lib.wasm stub");
        println!("cargo:warning=using stubbed profiler_lib.wasm for static analysis purposes...");
        return;
    }

    // When building the profiler crate explicitly fail if the
    // profiler library is not present.
    panic!(
        "javy_profiler_lib.wasm not found at {}.\n \
         Build with make cli features=profiler.",
        lib_path.display()
    );
}

fn is_clippy() -> bool {
    env::var("CARGO_CFG_FEATURE").as_deref() == Ok("cargo-clippy")
}
