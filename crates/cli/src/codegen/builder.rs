use crate::{
    codegen::{CodeGen, CodeGenType, DynamicGenerator, StaticGenerator},
    providers::Provider,
};
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
    pub fn build<T>(self, js_runtime_config: Config) -> Result<Box<dyn CodeGen>>
    where
        T: CodeGen,
    {
        match T::classify() {
            CodeGenType::Static => self.build_static(js_runtime_config),
            CodeGenType::Dynamic => {
                if js_runtime_config != Config::default() {
                    bail!("Cannot set JS runtime options when building a dynamic module")
                }
                self.build_dynamic()
            }
        }
    }

    fn build_static(self, js_runtime_config: Config) -> Result<Box<dyn CodeGen>> {
        let mut static_gen = Box::new(StaticGenerator::new(js_runtime_config));

        static_gen.source_compression = self.source_compression;
        static_gen.wit_opts = self.wit_opts;

        Ok(static_gen)
    }

    fn build_dynamic(self) -> Result<Box<dyn CodeGen>> {
        let mut dynamic_gen = Box::new(DynamicGenerator::new());
        dynamic_gen.source_compression = self.source_compression;
        dynamic_gen.provider = self.provider;
        dynamic_gen.wit_opts = self.wit_opts;
        Ok(dynamic_gen)
    }
}
