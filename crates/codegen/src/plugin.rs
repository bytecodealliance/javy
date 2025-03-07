use anyhow::{anyhow, Result};
use std::{borrow::Cow, fs, path::Path, str};

use super::bytecode;

/// The kind of a plugin.
// This is an internal detail of this module.
#[derive(Default, PartialEq, Copy, Clone)]
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
                // The import namespace to use for this plugin.
                let module = walrus::Module::from_buffer(plugin.as_bytes())?;
                let import_namespace: std::borrow::Cow<'_, [u8]> = module
                    .customs
                    .iter()
                    .find_map(|(_, section)| {
                        if section.name() == "import_namespace" {
                            Some(section)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow!("Plugin is missing import_namespace custom section"))?
                    .data(&Default::default()); // Argument is required but not actually used for anything.
                Ok(str::from_utf8(&import_namespace)?.to_string())
            }
        }
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
        bytecode::compile_source(self.as_bytes(), js_source_code)
    }
}
