use crate::{
    bytecode,
    codegen::{CodeGen, CodeGenType, DynamicGenerator, StaticGenerator},
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

/// Strategy for determining the import namespace to use.
pub(crate) enum ImportNamespace {
    /// Use javy_quickjs_provider_v2.
    JavyQuickJsProviderV2,
    /// Get the import namespace from the provider.
    FromProvider,
}

impl Default for ImportNamespace {
    fn default() -> Self {
        Self::FromProvider
    }
}

/// A code generation builder.
#[derive(Default)]
pub(crate) struct CodeGenBuilder {
    /// The import namespace for dynamically linked modules.
    import_namespace: Option<ImportNamespace>,
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

    /// Set the import namespace.
    pub fn import_namespace(&mut self, n: ImportNamespace) -> &mut Self {
        self.import_namespace = Some(n);
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

        dynamic_gen.import_namespace = match self.import_namespace {
            Some(ImportNamespace::JavyQuickJsProviderV2) => "javy_quickjs_provider_v2".to_string(),
            Some(ImportNamespace::FromProvider) => {
                bytecode::derive_import_namespace_from_provider()?
            }
            None => bail!("Import namespace not specified"),
        };

        dynamic_gen.wit_opts = self.wit_opts;

        Ok(dynamic_gen)
    }
}
