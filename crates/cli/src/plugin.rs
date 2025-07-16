use anyhow::{bail, Result};
use javy_codegen::Plugin;
use std::str;
use walrus::{ExportItem, ValType};

pub const PLUGIN_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.wasm"));
pub const QUICKJS_PROVIDER_V2_MODULE: &[u8] = include_bytes!("./javy_quickjs_provider_v2.wasm");

/// Represents the kind of a plugin.
// This is an internal detail of this module.
pub(crate) enum PluginKind {
    User,
    Default,
}

/// Represents a Plugin as well as it's kind
/// for use within the Javy CLI crate.
// This is an internal detail of this module.
pub(crate) struct CliPlugin {
    pub(crate) plugin: Plugin,
    pub(crate) kind: PluginKind,
}

impl CliPlugin {
    pub fn new(plugin: Plugin, kind: PluginKind) -> Self {
        CliPlugin { plugin, kind }
    }

    pub fn as_plugin(&self) -> &Plugin {
        &self.plugin
    }

    pub fn into_plugin(self) -> Plugin {
        self.plugin
    }
}

/// A validated but uninitialized plugin.
pub(crate) struct UninitializedPlugin<'a> {
    bytes: &'a [u8],
}

impl<'a> UninitializedPlugin<'a> {
    /// Creates a validated but uninitialized plugin.
    pub(crate) fn new(bytes: &'a [u8]) -> Result<Self> {
        Self::validate(bytes)?;
        Ok(Self { bytes })
    }

    /// Initializes the plugin.
    pub(crate) fn initialize(&self) -> Result<Vec<u8>> {
        javy_plugin_processing::initialize_plugin(self.bytes)
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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use walrus::{FunctionBuilder, ModuleConfig, ValType};

    use crate::plugin::UninitializedPlugin;

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
