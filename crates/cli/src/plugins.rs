use crate::bytecode;
use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use std::{
    fs,
    io::{Read, Seek},
    path::Path,
    str,
};
use walrus::{ExportItem, ValType};
use wasi_common::{pipe::WritePipe, sync::WasiCtxBuilder};
use wasmtime::{AsContextMut, Engine, Linker};
use wizer::Wizer;

const PLUGIN_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.wasm"));

/// Use the legacy plugin when using the `compile -d` command.
const QUICKJS_PROVIDER_V2_MODULE: &[u8] = include_bytes!("./javy_quickjs_provider_v2.wasm");

/// A property that is in the config schema returned by the plugin.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JsConfigProperty {
    /// The name of the property (e.g., `simd-json-builtins`).
    pub(crate) name: String,
    /// The documentation to display for the property.
    pub(crate) doc: String,
}

/// Different plugins that are available.
#[derive(Debug)]
pub enum Plugin {
    /// The default plugin.
    Default,
    /// A plugin for use with the `compile` to maintain backward compatibility.
    V2,
    /// A plugin provided by the user.
    User { bytes: Vec<u8> },
}

impl Default for Plugin {
    fn default() -> Self {
        Self::Default
    }
}

impl Plugin {
    /// Creates a new user plugin.
    pub fn new_user_plugin(path: &Path) -> Result<Self> {
        Ok(Self::User {
            bytes: fs::read(path)?,
        })
    }

    /// Returns true if the plugin is a user plugin.
    pub fn is_user_plugin(&self) -> bool {
        matches!(&self, Plugin::User { .. })
    }

    /// Returns true if the plugin is the legacy v2 plugin.
    pub fn is_v2_plugin(&self) -> bool {
        matches!(&self, Plugin::V2)
    }

    /// Returns the plugin Wasm module as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Default => PLUGIN_MODULE,
            Self::V2 => QUICKJS_PROVIDER_V2_MODULE,
            Self::User { bytes } => bytes,
        }
    }

    /// Uses the plugin to generate QuickJS bytecode.
    pub fn compile_source(&self, js_source_code: &[u8]) -> Result<Vec<u8>> {
        bytecode::compile_source(self, js_source_code)
    }

    /// The import namespace to use for this plugin.
    pub fn import_namespace(&self) -> Result<String> {
        match self {
            Self::V2 => Ok("javy_quickjs_provider_v2".to_string()),
            Self::Default | Self::User { .. } => {
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
                    .ok_or_else(|| anyhow!("Plugin is missing import_namespace custom section"))?
                    .data(&Default::default()); // Argument is required but not actually used for anything.
                Ok(str::from_utf8(&import_namespace)?.to_string())
            }
        }
    }

    /// The JS configuration properties supported by this plugin.
    pub fn config_schema(&self) -> Result<Vec<JsConfigProperty>> {
        match self {
            Self::V2 | Self::User { .. } => Ok(vec![]),
            Self::Default => {
                let engine = Engine::default();
                let module = wasmtime::Module::new(&engine, self.as_bytes())?;
                let mut linker = Linker::new(&engine);
                wasi_common::sync::snapshots::preview_1::add_wasi_snapshot_preview1_to_linker(
                    &mut linker,
                    |s| s,
                )?;
                let stdout = WritePipe::new_in_memory();
                let wasi = WasiCtxBuilder::new()
                    .inherit_stderr()
                    .stdout(Box::new(stdout.clone()))
                    .build();
                let mut store = wasmtime::Store::new(&engine, wasi);
                let instance = linker.instantiate(store.as_context_mut(), &module)?;
                instance
                    .get_typed_func::<(), ()>(store.as_context_mut(), "config_schema")?
                    .call(store.as_context_mut(), ())?;
                drop(store);
                let mut config_json = vec![];
                let mut cursor = stdout.try_into_inner().unwrap();
                cursor.rewind()?;
                cursor.read_to_end(&mut config_json)?;
                let config_schema = serde_json::from_slice::<ConfigSchema>(&config_json)?;
                let mut configs = Vec::with_capacity(config_schema.supported_properties.len());
                for config in config_schema.supported_properties {
                    configs.push(JsConfigProperty {
                        name: config.name,
                        doc: config.doc,
                    });
                }
                Ok(configs)
            }
        }
    }
}

/// A validated but uninitialized plugin.
pub(super) struct UninitializedPlugin<'a> {
    bytes: &'a [u8],
}

impl<'a> UninitializedPlugin<'a> {
    /// Creates a validated but uninitialized plugin.
    pub fn new(bytes: &'a [u8]) -> Result<Self> {
        Self::validate(bytes)?;
        Ok(Self { bytes })
    }

    fn validate(plugin_bytes: &'a [u8]) -> Result<()> {
        let mut errors = vec![];

        let module = walrus::Module::from_buffer(plugin_bytes)?;

        if let Err(err) = Self::validate_exported_func(&module, "initialize_runtime", &[], &[]) {
            errors.push(err);
        }
        if let Err(err) = Self::validate_exported_func(
            &module,
            "compile_src",
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        ) {
            errors.push(err);
        }
        if let Err(err) = Self::validate_exported_func(
            &module,
            "invoke",
            &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            &[],
        ) {
            errors.push(err);
        }

        let has_memory = module
            .exports
            .iter()
            .any(|export| export.name == "memory" && matches!(export.item, ExportItem::Memory(_)));
        if !has_memory {
            errors.push("missing exported memory named `memory`".to_string());
        }

        let has_import_namespace = module
            .customs
            .iter()
            .any(|(_, section)| section.name() == "import_namespace");
        if !has_import_namespace {
            errors.push("missing custom section named `import_namespace`".to_string());
        }

        if !errors.is_empty() {
            bail!("Problems with module: {}", errors.join(", "))
        }
        Ok(())
    }

    /// Initializes the plugin.
    pub fn initialize(&self) -> Result<Vec<u8>> {
        let initialized_plugin = Wizer::new()
            .allow_wasi(true)?
            .init_func("initialize_runtime")
            .keep_init_func(true)
            .wasm_bulk_memory(true)
            .run(self.bytes)?;

        let tempdir = tempfile::tempdir()?;
        let in_tempfile_path = tempdir.path().join("in_temp.wasm");
        let out_tempfile_path = tempdir.path().join("out_temp.wasm");
        fs::write(&in_tempfile_path, initialized_plugin)?;
        wasm_opt::OptimizationOptions::new_opt_level_3() // Aggressively optimize for speed.
            .shrink_level(wasm_opt::ShrinkLevel::Level0) // Don't optimize for size at the expense of performance.
            .debug_info(false)
            .run(&in_tempfile_path, &out_tempfile_path)?;
        Ok(fs::read(out_tempfile_path)?)
    }

    fn validate_exported_func(
        module: &walrus::Module,
        name: &str,
        expected_params: &[ValType],
        expected_results: &[ValType],
    ) -> Result<(), String> {
        let func_id = module
            .exports
            .get_func(name)
            .map_err(|_| format!("missing export for function named `{name}`"))?;
        let function = module.funcs.get(func_id);
        let ty_id = function.ty();
        let ty = module.types.get(ty_id);
        let params = ty.params();
        let has_correct_params = params == expected_params;
        let results = ty.results();
        let has_correct_results = results == expected_results;
        if !has_correct_params || !has_correct_results {
            return Err(format!("type for function `{name}` is incorrect"));
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigSchema {
    supported_properties: Vec<JsConfigProperty>,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use walrus::{FunctionBuilder, ModuleConfig, ValType};

    use crate::plugins::UninitializedPlugin;

    #[test]
    fn test_validate_plugin_with_everything_missing() -> Result<()> {
        let mut empty_module = walrus::Module::with_config(ModuleConfig::default());
        let plugin_bytes = empty_module.emit_wasm();
        let error = UninitializedPlugin::new(&plugin_bytes).err().unwrap();
        assert_eq!(
            error.to_string(),
            "Problems with module: missing export for function named \
            `initialize_runtime`, missing export for function named \
            `compile_src`, missing export for function named `invoke`, \
            missing exported memory named `memory`, missing custom section \
            named `import_namespace`"
        );
        Ok(())
    }

    #[test]
    fn test_validate_plugin_with_wrong_params_for_initialize_runtime() -> Result<()> {
        let mut module = walrus::Module::with_config(ModuleConfig::default());
        let initialize_runtime = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[])
            .finish(vec![], &mut module.funcs);
        module.exports.add("initialize_runtime", initialize_runtime);

        let plugin_bytes = module.emit_wasm();
        let error = UninitializedPlugin::new(&plugin_bytes).err().unwrap();
        let expected_part_of_error =
            "Problems with module: type for function `initialize_runtime` is incorrect,";
        if !error.to_string().contains(expected_part_of_error) {
            panic!("Expected error to contain '{expected_part_of_error}' but it did not. Full error is: '{error}'");
        }
        Ok(())
    }

    #[test]
    fn test_validate_plugin_with_wrong_results_for_initialize_runtime() -> Result<()> {
        let mut module = walrus::Module::with_config(ModuleConfig::default());
        let mut initialize_runtime = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);
        initialize_runtime.func_body().i32_const(0);
        let initialize_runtime = initialize_runtime.finish(vec![], &mut module.funcs);
        module.exports.add("initialize_runtime", initialize_runtime);

        let plugin_bytes = module.emit_wasm();
        let error = UninitializedPlugin::new(&plugin_bytes).err().unwrap();
        let expected_part_of_error =
            "Problems with module: type for function `initialize_runtime` is incorrect,";
        if !error.to_string().contains(expected_part_of_error) {
            panic!("Expected error to contain '{expected_part_of_error}' but it did not. Full error is: '{error}'");
        }
        Ok(())
    }
}
