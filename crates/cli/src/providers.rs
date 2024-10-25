use crate::bytecode;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{
    borrow::Cow,
    fs,
    io::{Read, Seek},
    path::PathBuf,
    str,
};
use wasi_common::{pipe::WritePipe, sync::WasiCtxBuilder};
use wasmtime::{AsContextMut, Engine, Linker, Module, Store};

const QUICKJS_PROVIDER_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/provider.wasm"));

/// Use the legacy provider when using the `compile -d` command.
const QUICKJS_PROVIDER_V2_MODULE: &[u8] = include_bytes!("./javy_quickjs_provider_v2.wasm");

/// Different providers that are available.
#[derive(Debug)]
pub enum Provider {
    /// The default provider.
    Default,
    /// A provider for use with the `compile` to maintain backward compatibility.
    V2,
    UserPlugin {
        path: PathBuf,
    },
}

impl Default for Provider {
    fn default() -> Self {
        Self::Default
    }
}

impl Provider {
    /// Returns the provider Wasm module as a byte slice.
    pub fn as_bytes(&self) -> Cow<'_, [u8]> {
        match self {
            Self::Default => Cow::Borrowed(QUICKJS_PROVIDER_MODULE),
            Self::V2 => Cow::Borrowed(QUICKJS_PROVIDER_V2_MODULE),
            Self::UserPlugin { path } => Cow::Owned(fs::read(path).unwrap()),
        }
    }

    /// Uses the provider to generate QuickJS bytecode.
    pub fn compile_source(&self, js_source_code: &[u8]) -> Result<Vec<u8>> {
        bytecode::compile_source(self, js_source_code)
    }

    /// The import namespace to use for this provider.
    pub fn import_namespace(&self) -> Result<String> {
        match self {
            Self::V2 => Ok("javy_quickjs_provider_v2".to_string()),
            Self::Default => {
                let module = walrus::Module::from_buffer(&self.as_bytes())?;
                let import_namespace = module
                    .customs
                    .iter()
                    .find_map(|(_, section)| {
                        if section.name() == "import_namespace" {
                            Some(section)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow!("Provider is missing import_namespace custom section"))?
                    .data(&Default::default()); // Argument is required but not actually used for anything.
                Ok(str::from_utf8(&import_namespace)?.to_string())
            }
            Self::UserPlugin { path: _ } => todo!(),
        }
    }

    pub fn support_config(&self) -> Result<Vec<(String, String, String)>> {
        let engine = Engine::default();
        let module = Module::new(&engine, self.as_bytes())?;
        let mut linker = Linker::new(&engine);
        wasi_common::sync::snapshots::preview_1::add_wasi_snapshot_preview1_to_linker(
            &mut linker,
            |s| s,
        )?;
        let stdout = WritePipe::new_in_memory();
        let wasi = WasiCtxBuilder::new()
            .inherit_stderr()
            .stdout(Box::new(stdout.clone()))
            .build();
        let mut store = Store::new(&engine, wasi);
        let instance = linker.instantiate(store.as_context_mut(), &module)?;
        instance
            .get_typed_func::<(), ()>(store.as_context_mut(), "supported_config")?
            .call(store.as_context_mut(), ())?;
        drop(store);
        let mut config_json = vec![];
        let mut cursor = stdout.try_into_inner().unwrap();
        cursor.rewind()?;
        cursor.read_to_end(&mut config_json)?;
        let supported_configs = serde_json::from_slice::<SupportedConfigs>(&config_json)?;
        let mut configs = Vec::with_capacity(supported_configs.supported_properties.len());
        for config in supported_configs.supported_properties {
            configs.push((config.name, config.help, config.doc));
        }
        Ok(configs)
    }
}

#[derive(Debug, Deserialize)]
struct SupportedConfigs {
    supported_properties: Vec<SupportedConfig>,
}

#[derive(Debug, Deserialize)]
struct SupportedConfig {
    name: String,
    help: String,
    doc: String,
}
