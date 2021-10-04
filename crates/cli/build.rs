use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    if let Ok("cargo-clippy") = env::var("CARGO_CFG_FEATURE").as_ref().map(String::as_str) {
        stub_engine_for_clippy();
    } else {
        copy_engine_binary();
    }
}

// When using clippy, we need to write a stubbed engine.wasm file to ensure compilation succeeds. This
// skips building the actual engine.wasm binary that would be injected into the CLI binary.
fn stub_engine_for_clippy() {
    let engine_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("engine.wasm");

    if !engine_path.exists() {
        std::fs::write(engine_path, &[]).expect("failed to write empty engine.wasm stub");
        println!("cargo:warning=using stubbed engine.wasm for static analysis purposes...");
    }
}

// Copy the engine binary build from the `core` crate
fn copy_engine_binary() {
    let override_engine_path = env::var("JAVY_ENGINE_PATH");
    let is_override = override_engine_path.is_ok();
    let mut engine_path = PathBuf::from(
        override_engine_path.unwrap_or_else(|_| env::var("CARGO_MANIFEST_DIR").unwrap()),
    );

    if !is_override {
        engine_path.pop();
        engine_path.pop();
        engine_path = engine_path.join("target/wasm32-wasi/release/javy_core.wasm");
    }

    println!("cargo:rerun-if-changed={:?}", engine_path);
    println!("cargo:rerun-if-changed=build.rs");

    if engine_path.exists() {
        let copied_engine_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("engine.wasm");

        fs::copy(&engine_path, &copied_engine_path).unwrap();
    }
}
