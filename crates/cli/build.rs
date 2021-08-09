use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir: PathBuf = env::var("OUT_DIR")
        .expect("failed to retrieve out dir")
        .into();

    // When using clippy, we need to write a stubbed engine.wasm file to ensure compilation succeeds.
    if let Ok("cargo-clippy") = env::var("CARGO_CFG_FEATURE").as_ref().map(String::as_str) {
        std::fs::write(out_dir.join("engine.wasm"), "")
            .expect("failed to write empty engine.wasm stub");
        std::process::exit(0);
    }

    let engine_path: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok()
        .map(|mut b| {
            b.pop();
            b.pop();
            b.join("target")
                .join("wasm32-wasi")
                .join("release")
                .join("javy_core.wasm")
        })
        .expect("failed to create path");

    println!("cargo:rerun-if-changed={:?}", engine_path);

    // Only copy the file when it exists. Cargo will take care of re-running this script when the file changes.
    if engine_path.exists() {
        std::fs::copy(&engine_path, out_dir.join("engine.wasm"))
            .unwrap_or_else(|_| panic!("failed to copy engine from {:?}", engine_path));
    }
}
