use crate::bytecode;
use anyhow::{anyhow, Result};
use std::str;

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
}
