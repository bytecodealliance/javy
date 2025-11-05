use std::env;
use std::fs;
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

    let initialized_plugin = javy_plugin_processing::initialize_plugin(&fs::read(&plugin_path)?)?;
    fs::write(&plugin_wizened_path, &initialized_plugin)?;

    println!("cargo:rerun-if-changed={}", plugin_path.to_str().unwrap());
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR")?;
    let copied_plugin_path = Path::new(&out_dir).join("plugin.wasm");

    fs::copy(&plugin_wizened_path, copied_plugin_path)?;
    Ok(())
}
