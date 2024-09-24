use crate::codegen::{CodeGen, Generator};
use anyhow::{bail, Result};
use javy_config::Config;
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
    /// The QuickJS provider module version.
    provider_version: Option<&'static str>,
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

    /// Set the provider version.
    pub fn provider_version(&mut self, v: &'static str) -> &mut Self {
        self.provider_version = Some(v);
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

    pub fn build_static(
        self,
        js_runtime_config: Config,
        plugin: Option<PathBuf>,
    ) -> Result<Box<dyn CodeGen>> {
        Ok(Box::new(Generator::new(
            super::LinkingStrategy::Static {
                js_runtime_config,
                plugin,
            },
            self.wit_opts,
        )))
    }

    pub fn build_dynamic(self, import_namespace: Option<String>) -> Result<Box<dyn CodeGen>> {
        Ok(Box::new(Generator::new(
            super::LinkingStrategy::Dynamic {
                import_namespace: import_namespace.unwrap(),
            },
            self.wit_opts,
        )))
    }
}
