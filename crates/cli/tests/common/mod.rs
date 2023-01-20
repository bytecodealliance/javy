use anyhow::Result;
use std::path::{Path, PathBuf};
use wasmtime::{Engine, Module};

pub fn create_quickjs_provider_module(engine: &Engine) -> Result<Module> {
    let mut lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lib_path.pop();
    lib_path.pop();
    lib_path = lib_path.join(
        Path::new("target")
            .join("wasm32-wasi")
            .join("release")
            .join("javy_quickjs_provider_wizened.wasm"),
    );
    Module::from_file(engine, lib_path)
}
