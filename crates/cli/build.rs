use std::env;
use std::fs;
use std::io::{Read, Write};

use std::path::{Path, PathBuf};

use anyhow::Result;

fn main() -> Result<()> {
    if let Ok("cargo-clippy") = env::var("CARGO_CFG_FEATURE").as_ref().map(String::as_str) {
        stub_plugin_for_clippy()
    } else {
        copy_plugin()
    }
}

// When using clippy, we need to write stubbed plugin.wasm file to ensure
// compilation succeeds. This skips building the actual plugin.wasm that would
// be injected into the CLI binary.
fn stub_plugin_for_clippy() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let plugin_path = out_dir.join("plugin.wasm");

    if !plugin_path.exists() {
        std::fs::write(plugin_path, [])?;
        println!("cargo:warning=using stubbed plugin.wasm for static analysis purposes...");
    }
    Ok(())
}

fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![];
    fs::File::open(path.as_ref())?.read_to_end(&mut buf)?;
    Ok(buf)
}

// Copy the plugin binary build from the `plugin` crate
fn copy_plugin() -> Result<()> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let module_path = PathBuf::from(&cargo_manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/wasm32-wasip1/release");
    let plugin_path = module_path.join("plugin.wasm");
    let plugin_wizened_path = module_path.join("plugin_wizened.wasm");

    let mut wizer = wizer::Wizer::new();
    let wizened = wizer
        .init_func("initialize_runtime")
        .keep_init_func(true) // Necessary for static codegen.
        .allow_wasi(true)?
        .wasm_bulk_memory(true)
        .run(read_file(&plugin_path)?.as_slice())?;
    fs::File::create(&plugin_wizened_path)?.write_all(&wizened)?;

    println!("cargo:rerun-if-changed={}", plugin_path.to_str().unwrap());
    println!("cargo:rerun-if-changed=build.rs");

    if plugin_wizened_path.exists() {
        let out_dir = env::var("OUT_DIR")?;
        let copied_plugin_path = Path::new(&out_dir).join("plugin.wasm");

        fs::copy(&plugin_wizened_path, copied_plugin_path)?;
    }
    Ok(())
}
