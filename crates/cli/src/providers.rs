use crate::bytecode;
use anyhow::Result;

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
    pub fn import_namespace(&self) -> String {
        let prefix = "javy_quickjs_provider_v";
        let version = match self {
            Self::Default => 3,
            Self::V2 => 2,
        };
        format!("{prefix}{version}")
    }
}
