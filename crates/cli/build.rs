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

// When using clippy, we need to write stubbed provider.wasm file to ensure
// compilation succeeds. This skips building the actual provider.wasm that would
// be injected into the CLI binary.
fn stub_javy_core_for_clippy() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let provider_path = out_dir.join("provider.wasm");

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
    let quickjs_provider_path = module_path.join("javy_quickjs_provider.wasm");
    let quickjs_provider_wizened_path = module_path.join("javy_quickjs_provider_wizened.wasm");

    let mut wizer = wizer::Wizer::new();
    let wizened = wizer
        .keep_init_func(true)
        .init_func("initialize_runtime")
        .allow_wasi(true)?
        .wasm_bulk_memory(true)
        .run(read_file(&quickjs_provider_path)?.as_slice())?;
    fs::File::create(&quickjs_provider_wizened_path)?.write_all(&wizened)?;

    println!(
        "cargo:rerun-if-changed={}",
        quickjs_provider_path.to_str().unwrap()
    );
    println!("cargo:rerun-if-changed=build.rs");

    if quickjs_provider_wizened_path.exists() {
        let out_dir = env::var("OUT_DIR")?;
        let copied_provider_path = Path::new(&out_dir).join("provider.wasm");
        fs::copy(&quickjs_provider_wizened_path, copied_provider_path)?;
    }
    Ok(())
}
