use crate::{
    codegen::{CodeGenType, Generator},
    commands::RuntimeConfig,
    providers::Provider,
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
#[derive(Default)]
pub(crate) struct CodeGenBuilder {
    /// The provider to use.
    provider: Provider,
    /// WIT options for code generation.
    wit_opts: WitOptions,
    /// Whether to compress the original JS source.
    source_compression: bool,
}

impl CodeGenBuilder {
    /// Create a new [`CodeGenBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the provider.
    pub fn provider(&mut self, provider: Provider) -> &mut Self {
        self.provider = provider;
        self
    }

    /// Set the wit options.
    pub fn wit_opts(&mut self, opts: WitOptions) -> &mut Self {
        self.wit_opts = opts;
        self
    }

    /// Whether to compress the JS source.
    pub fn source_compression(&mut self, compress: bool) -> &mut Self {
        self.source_compression = compress;
        self
    }

    /// Build a [`CodeGenerator`].
    pub fn build(self, ty: CodeGenType, js_runtime_config: RuntimeConfig) -> Result<Generator> {
        if let CodeGenType::Dynamic = ty {
            if js_runtime_config.has_config() {
                bail!("Cannot set JS runtime options when building a dynamic module");
            }
        }
        let mut generator = Generator::new(ty, js_runtime_config, self.provider);
        generator.source_compression = self.source_compression;
        generator.wit_opts = self.wit_opts;
        Ok(generator)
    }
}
