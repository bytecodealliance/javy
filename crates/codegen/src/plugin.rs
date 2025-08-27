use anyhow::{anyhow, bail, Result};
use std::{borrow::Cow, fs, path::Path, str};
use walrus::{ExportItem, ValType};
use wasmparser::Parser;

/// The kind of a plugin.
// This is an internal detail of this module.
#[derive(Debug, Default, PartialEq, Copy, Clone)]
#[allow(dead_code)] // Suppresses warnings for feature-gated variants
pub(crate) enum PluginKind {
    #[default]
    User,
    V2,
}

impl PluginKind {
    /// Determine the import namespace of a provided plugin.
    pub(crate) fn import_namespace(self, plugin: &Plugin) -> Result<String> {
        match self {
            PluginKind::V2 => Ok("javy_quickjs_provider_v2".to_string()),
            PluginKind::User => {
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

    pub(crate) fn realloc_fn_name(self) -> &'static str {
        if self == Self::V2 {
            "canonical_abi_realloc"
        } else {
            "cabi_realloc"
        }
    }

    pub(crate) fn compile_fn_name(self) -> &'static str {
        if self == Self::V2 {
            "compile_src"
        } else {
            "compile-src"
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
    pub fn new(bytes: Cow<'static, [u8]>) -> Result<Self> {
        Self::validate(&bytes)?;
        Ok(Self { bytes })
    }

    #[cfg(feature = "plugin_internal")]
    pub fn new_javy_quickjs_v2_plugin(bytes: &'static [u8]) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    /// Constructs a new [`Plugin`] from a given path.
    pub fn new_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = fs::read(path)?;
        Self::new(bytes.into())
    }

    /// Returns the [`Plugin`] as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Validates if `plugin_bytes` are a valid plugin.
    pub fn validate(plugin_bytes: &[u8]) -> Result<()> {
        if !Parser::is_core_wasm(plugin_bytes) {
            bail!("Could not process plugin: Expected Wasm module, received unknown file type");
        }

        let mut errors = vec![];

        let module = walrus::Module::from_buffer(plugin_bytes)?;

        if module.exports.get_func("compile_src").is_ok() {
            bail!("Could not process plugin: Using unsupported legacy plugin API");
        }

        if let Err(err) = validate_exported_func(&module, "initialize-runtime", &[], &[]) {
            errors.push(err);
        }
        if let Err(err) = validate_exported_func(
            &module,
            "compile-src",
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        ) {
            errors.push(err);
        }
        if let Err(err) = validate_exported_func(
            &module,
            "invoke",
            &[
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
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
            bail!("Could not process plugin: {}", errors.join(", "))
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use walrus::{FunctionBuilder, ModuleConfig, ValType};

    use crate::Plugin;

    #[test]
    fn test_validate_plugin_with_empty_file() -> Result<()> {
        let err = Plugin::new(vec![].into()).err().unwrap();
        assert_eq!(
            err.to_string(),
            "Could not process plugin: Expected Wasm module, received unknown file type"
        );
        Ok(())
    }

    #[test]
    fn test_validate_plugin_with_old_plugin() -> Result<()> {
        let mut module = walrus::Module::with_config(ModuleConfig::default());
        module.add_import_memory("foo", "memory", false, false, 0, None, None);
        let mut compile_src_fn = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        compile_src_fn.func_body().unreachable();
        let compile_src_fn = compile_src_fn.finish(vec![], &mut module.funcs);
        module.exports.add("compile_src", compile_src_fn);

        let err = Plugin::new(module.emit_wasm().into()).err().unwrap();
        assert_eq!(
            err.to_string(),
            "Could not process plugin: Using unsupported legacy plugin API"
        );
        Ok(())
    }

    #[test]
    fn test_validate_plugin_with_incorrect_invoke_and_everything_missing() -> Result<()> {
        let mut module = walrus::Module::with_config(ModuleConfig::default());
        let invoke = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            &[],
        )
        .finish(vec![], &mut module.funcs);
        module.exports.add("invoke", invoke);

        let plugin_bytes = module.emit_wasm();
        let error = Plugin::validate(&plugin_bytes).err().unwrap();
        assert_eq!(
            error.to_string(),
            "Could not process plugin: missing export for function named \
            `initialize-runtime`, missing export for function named \
            `compile-src`, type for function `invoke` is incorrect, missing \
            exported memory named `memory`, missing custom section named \
            `import_namespace`"
        );
        Ok(())
    }

    #[test]
    fn test_validate_plugin_with_everything_missing() -> Result<()> {
        let mut empty_module = walrus::Module::with_config(ModuleConfig::default());
        let plugin_bytes = empty_module.emit_wasm();
        let error = Plugin::new(plugin_bytes.into()).err().unwrap();
        assert_eq!(
            error.to_string(),
            "Could not process plugin: missing export for function named \
            `initialize-runtime`, missing export for function named \
            `compile-src`, missing export for function named `invoke`, \
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
        module.exports.add("initialize-runtime", initialize_runtime);

        let plugin_bytes = module.emit_wasm();
        let error = Plugin::new(plugin_bytes.into()).err().unwrap();
        let expected_part_of_error =
            "Could not process plugin: type for function `initialize-runtime` is incorrect,";
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
        module.exports.add("initialize-runtime", initialize_runtime);

        let plugin_bytes = module.emit_wasm();
        let error = Plugin::new(plugin_bytes.into()).err().unwrap();
        let expected_part_of_error =
            "Could not process plugin: type for function `initialize-runtime` is incorrect,";
        if !error.to_string().contains(expected_part_of_error) {
            panic!("Expected error to contain '{expected_part_of_error}' but it did not. Full error is: '{error}'");
        }
        Ok(())
    }
}
