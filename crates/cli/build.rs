use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Ok("cargo-clippy") = env::var("CARGO_CFG_FEATURE").as_ref().map(String::as_str) {
        stub_javy_core_for_clippy();
    } else {
        copy_javy_core();
    }
}

// When using clippy, we need to write stubbed engine.wasm and provider.wasm files to ensure
// compilation succeeds. This skips building the actual engine.wasm and provider.wasm that would
// be injected into the CLI binary.
fn stub_javy_core_for_clippy() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let engine_path = out_dir.join("engine.wasm");
    let provider_path = out_dir.join("provider.wasm");

    if !engine_path.exists() {
        std::fs::write(engine_path, []).expect("failed to write empty engine.wasm stub");
        println!("cargo:warning=using stubbed engine.wasm for static analysis purposes...");
    }

    if !provider_path.exists() {
        std::fs::write(provider_path, []).expect("failed to write empty provider.wasm stub");
        println!("cargo:warning=using stubbed provider.wasm for static analysis purposes...");
    }
}

// Copy the engine binary build from the `core` crate
fn copy_javy_core() {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let override_engine_path = env::var("JAVY_ENGINE_PATH");
    let is_override = override_engine_path.is_ok();
    let mut engine_path =
        PathBuf::from(override_engine_path.as_ref().unwrap_or(&cargo_manifest_dir));

    if !is_override {
        engine_path.pop();
        engine_path.pop();
        engine_path = engine_path.join("target/wasm32-wasi/release/javy_core.wasm");
    }

    let mut quickjs_provider_path = PathBuf::from(&cargo_manifest_dir);
    quickjs_provider_path.pop();
    quickjs_provider_path.pop();
    quickjs_provider_path =
        quickjs_provider_path.join("target/wasm32-wasi/release/javy_quickjs_provider.wasm");

    println!("cargo:rerun-if-changed={}", engine_path.to_str().unwrap());
    println!(
        "cargo:rerun-if-changed={}",
        quickjs_provider_path.to_str().unwrap()
    );
    println!("cargo:rerun-if-changed=build.rs");

    if engine_path.exists() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let copied_engine_path = Path::new(&out_dir).join("engine.wasm");
        let copied_provider_path = Path::new(&out_dir).join("provider.wasm");

        fs::copy(&engine_path, copied_engine_path).unwrap();
        fs::copy(&quickjs_provider_path, copied_provider_path).unwrap();
    }
}
