use std::env;
use std::fs;
use std::io::{Read, Write};

use std::path::{Path, PathBuf};

use anyhow::Result;

fn main() -> Result<()> {
    if let Ok("cargo-clippy") = env::var("CARGO_CFG_FEATURE").as_ref().map(String::as_str) {
        stub_javy_core_for_clippy()
    } else {
        copy_javy_core()
    }
}

// When using clippy, we need to write stubbed engine.wasm and provider.wasm files to ensure
// compilation succeeds. This skips building the actual engine.wasm and provider.wasm that would
// be injected into the CLI binary.
fn stub_javy_core_for_clippy() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let engine_path = out_dir.join("engine.wasm");
    let provider_path = out_dir.join("provider.wasm");

    if !engine_path.exists() {
        std::fs::write(engine_path, [])?;
        println!("cargo:warning=using stubbed engine.wasm for static analysis purposes...");
    }

    if !provider_path.exists() {
        std::fs::write(provider_path, [])?;
        println!("cargo:warning=using stubbed provider.wasm for static analysis purposes...");
    }
    Ok(())
}

fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![];
    fs::File::open(path.as_ref())?.read_to_end(&mut buf)?;
    Ok(buf)
}

// Copy the engine binary build from the `core` crate
fn copy_javy_core() -> Result<()> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let module_path = PathBuf::from(&cargo_manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/wasm32-wasi/release");
    let engine_path = module_path.join("javy_core.wasm");
    let quickjs_provider_path = module_path.join("javy_quickjs_provider.wasm");
    let quickjs_provider_wizened_path = module_path.join("javy_quickjs_provider_wizened.wasm");

    let mut wizer = wizer::Wizer::new();
    let wizened = wizer
        .allow_wasi(true)?
        .wasm_bulk_memory(true)
        .run(read_file(&quickjs_provider_path)?.as_slice())?;
    fs::File::create(&quickjs_provider_wizened_path)?.write_all(&wizened)?;

    println!("cargo:rerun-if-changed={}", engine_path.to_str().unwrap());
    println!(
        "cargo:rerun-if-changed={}",
        quickjs_provider_path.to_str().unwrap()
    );
    println!("cargo:rerun-if-changed=build.rs");

    if engine_path.exists() {
        let out_dir = env::var("OUT_DIR")?;
        let copied_engine_path = Path::new(&out_dir).join("engine.wasm");
        let copied_provider_path = Path::new(&out_dir).join("provider.wasm");

        fs::copy(&engine_path, copied_engine_path)?;
        fs::copy(&quickjs_provider_wizened_path, copied_provider_path)?;
    }
    Ok(())
}
