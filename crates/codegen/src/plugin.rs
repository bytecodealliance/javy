use anyhow::{bail, Result};
use std::{borrow::Cow, fs, path::Path};
use wasmparser::{Parser, Payload};

use super::bytecode;

/// The kind of a plugin.
// This is an internal detail of this module.
#[derive(Debug, Default, PartialEq, Copy, Clone)]
#[allow(dead_code)] // Suppresses warnings for feature-gated variants
pub(crate) enum PluginKind {
    #[default]
    User,
    Default,
    V2,
}

impl PluginKind {
    /// Determine the import namespace of a provided plugin.
    pub(crate) fn import_namespace(self, plugin: &Plugin) -> Result<String> {
        match self {
            PluginKind::V2 => Ok("javy_quickjs_provider_v2".to_string()),
            PluginKind::User | PluginKind::Default => {
                for payload in Parser::new(0).parse_all(plugin.as_bytes()) {
                    match payload? {
                        Payload::ExportSection(reader) => {
                            let export_prefix = reader.into_iter().find_map(|export| {
                                export.map_or_else(
                                    |_| None,
                                    |export| export.name.strip_suffix("#invoke"),
                                )
                            });
                            if let Some(export_prefix) = export_prefix {
                                return Ok(export_prefix.to_string());
                            }
                        }
                        _ => continue,
                    }
                }
                bail!("Plugin missing expected invoke export")
            }
        }
    }

    pub(crate) fn is_v2(self) -> bool {
        PluginKind::V2 == self
    }
}

/// A Javy plugin.
#[derive(Clone, Debug, Default)]
pub struct Plugin {
    bytes: Cow<'static, [u8]>,
}

impl Plugin {
    /// Constructs a new [`Plugin`].
    pub fn new(bytes: Cow<'static, [u8]>) -> Self {
        Plugin { bytes }
    }

    /// Constructs a new [`Plugin`] from a given path.
    pub fn new_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = fs::read(path)?;
        Ok(Self::new(bytes.into()))
    }

    /// Returns the [`Plugin`] as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Generate valid QuickJS bytecode from Javascript source code.
    pub(crate) fn compile_source(&self, js_source_code: &[u8]) -> Result<Vec<u8>> {
        bytecode::compile_source(self, js_source_code)
    }

    pub fn is_component(&self) -> Result<bool> {
        if self.bytes.len() < 8 {
            bail!("Plugin is not a valid Wasm module or component");
        }

        let version =
            u32::from_le_bytes([self.bytes[4], self.bytes[5], self.bytes[6], self.bytes[7]]);
        if version == 1 {
            Ok(false)
        } else if version == 0x0D {
            Ok(true)
        } else {
            bail!("Plugin is not a valid Wasm module or component");
        }
    }
}
