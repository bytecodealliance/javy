use crate::bytecode;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{
    io::{Read, Seek},
    str,
};
use wasi_common::{pipe::WritePipe, sync::WasiCtxBuilder};
use wasmtime::{AsContextMut, Engine, Linker};

const QUICKJS_PROVIDER_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/provider.wasm"));

/// Use the legacy provider when using the `compile -d` command.
const QUICKJS_PROVIDER_V2_MODULE: &[u8] = include_bytes!("./javy_quickjs_provider_v2.wasm");

/// A property that is in the config schema returned by the provider.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JsConfigProperty {
    /// The name of the property (e.g., `simd-json-builtins`).
    pub(crate) name: String,
    /// The documentation to display for the property.
    pub(crate) doc: String,
}

/// Different providers that are available.
#[derive(Debug)]
pub enum Provider {
    /// The default provider.
    Default,
    /// A provider for use with the `compile` to maintain backward compatibility.
    V2,
}

impl Default for Provider {
    fn default() -> Self {
        Self::Default
    }
}

impl Provider {
    /// Returns the provider Wasm module as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Default => QUICKJS_PROVIDER_MODULE,
            Self::V2 => QUICKJS_PROVIDER_V2_MODULE,
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
                let module = walrus::Module::from_buffer(self.as_bytes())?;
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
        }
    }

    /// The JS configuration properties supported by this provider.
    pub fn config_schema(&self) -> Result<Vec<JsConfigProperty>> {
        match self {
            Self::V2 => Ok(vec![]),
            Self::Default => {
                let engine = Engine::default();
                let module = wasmtime::Module::new(&engine, self.as_bytes())?;
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
                let mut store = wasmtime::Store::new(&engine, wasi);
                let instance = linker.instantiate(store.as_context_mut(), &module)?;
                instance
                    .get_typed_func::<(), ()>(store.as_context_mut(), "config_schema")?
                    .call(store.as_context_mut(), ())?;
                drop(store);
                let mut config_json = vec![];
                let mut cursor = stdout.try_into_inner().unwrap();
                cursor.rewind()?;
                cursor.read_to_end(&mut config_json)?;
                let config_schema = serde_json::from_slice::<ConfigSchema>(&config_json)?;
                let mut configs = Vec::with_capacity(config_schema.supported_properties.len());
                for config in config_schema.supported_properties {
                    configs.push(JsConfigProperty {
                        name: config.name,
                        doc: config.doc,
                    });
                }
                Ok(configs)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigSchema {
    supported_properties: Vec<JsConfigProperty>,
}
