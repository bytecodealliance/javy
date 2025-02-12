use anyhow::{anyhow, Result};
use std::{
    fs,
    io::{self},
    path::Path,
    str,
};

use super::bytecode;

/// Represents the kind of a plugin.
// This is an internal detail of this module.
#[derive(Default, PartialEq, Copy, Clone)]
pub(crate) enum PluginKind {
    #[default]
    User,
    Default,
    V2,
}

impl PluginKind {
    /// Determine the import namespace of a provided plugin.
    // This is an internal detail of this module.
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

/// Represents any valid Javy plugin.
#[derive(Clone, Debug)]
pub struct Plugin {
    bytes: Vec<u8>,
}

impl Default for Plugin {
    fn default() -> Self {
        Self { bytes: Vec::new() }
    }
}

impl Plugin {
    /// Constructs a new instance of Plugin.
    pub fn new(bytes: Vec<u8>) -> Self {
        Plugin { bytes: bytes }
    }

    /// Constructs a new instance of Plugin from a given path.
    pub fn new_from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        Ok(Self::new(bytes))
    }

    /// Returns the plugin Wasm module as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl Plugin {
    /// Generate valid QuickJS bytecode using a Plugin.
    pub(crate) fn compile_source(&self, js_source_code: &[u8]) -> Result<Vec<u8>> {
        bytecode::compile_source(&self.as_bytes(), js_source_code)
    }
}
