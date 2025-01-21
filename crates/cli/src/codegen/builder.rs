use crate::{
    codegen::{CodeGenType, Generator},
    plugins::Plugin,
};
use anyhow::{bail, Result};
use std::path::PathBuf;

/// Options for using WIT in the code generation process.
#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct WitOptions {
    /// The path of the .wit file to use.
    pub path: Option<PathBuf>,
    /// The name of the wit world to use.
    pub world: Option<String>,
}

impl WitOptions {
    pub fn from_tuple(opts: (Option<PathBuf>, Option<String>)) -> Result<Self> {
        match opts {
            (None, None) => Ok(Self {
                path: None,
                world: None,
            }),
            (None, Some(_)) => Ok(Self {
                path: None,
                world: None,
            }),
            (Some(_), None) => bail!("Must provide WIT world when providing WIT file"),
            (path, world) => Ok(Self { path, world }),
        }
    }

    /// Whether WIT options were defined.
    pub fn defined(&self) -> bool {
        self.path.is_some() && self.world.is_some()
    }

    /// Unwraps a refernce to the .wit file path.
    pub fn unwrap_path(&self) -> &PathBuf {
        self.path.as_ref().unwrap()
    }

    /// Unwraps a reference to the WIT world name.
    pub fn unwrap_world(&self) -> &String {
        self.world.as_ref().unwrap()
    }
}

/// A code generation builder.
pub(crate) struct CodeGenBuilder {
    /// The plugin to use.
    plugin: Plugin,
    /// WIT options for code generation.
    wit_opts: WitOptions,
    /// Whether to compress the original JS source.
    source_compression: bool,
}

impl CodeGenBuilder {
    /// Create a new [`CodeGenBuilder`].
    pub fn new(plugin: Plugin, wit_opts: WitOptions, source_compression: bool) -> Self {
        Self {
            plugin,
            wit_opts,
            source_compression,
        }
    }
}

#[cfg(feature = "plugin-internal")]
impl CodeGenBuilder {
    /// Build a [`CodeGenerator`].
    pub fn build(self, ty: CodeGenType, js_runtime_config: Vec<u8>) -> Result<Generator> {
        let mut generator = Generator::new(ty, js_runtime_config, self.plugin);
        generator.source_compression = self.source_compression;
        generator.wit_opts = self.wit_opts;
        Ok(generator)
    }
}

#[cfg(not(feature = "plugin-internal"))]
impl CodeGenBuilder {
    /// Build a [`CodeGenerator`].
    pub fn build(self, ty: CodeGenType) -> Result<Generator> {
        let mut generator = Generator::new(ty, Vec::new(), self.plugin);
        generator.source_compression = self.source_compression;
        generator.wit_opts = self.wit_opts;
        Ok(generator)
    }
}
