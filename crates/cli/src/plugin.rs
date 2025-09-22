use anyhow::{bail, Result};
use javy_codegen::Plugin;

pub const PLUGIN_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.wasm"));

/// Represents the kind of a plugin.
// This is an internal detail of this module.
#[derive(Debug)]
pub(crate) enum PluginKind {
    User,
    Default,
}

/// Represents a Plugin as well as it's kind
/// for use within the Javy CLI crate.
// This is an internal detail of this module.
#[derive(Debug)]
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
#[derive(Debug)]
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
        let plugin_bytes = match javy_plugin_processing::extract_core_module(plugin_bytes) {
            Err(e) => bail!("Could not process plugin: {e}"),
            Ok(plugin_bytes) => plugin_bytes,
        };
        Plugin::validate(&plugin_bytes)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use walrus::ModuleConfig;
    use wit_component::ComponentEncoder;

    use crate::plugin::UninitializedPlugin;

    #[test]
    fn test_validate_plugin_with_empty_file() -> Result<()> {
        let error = UninitializedPlugin::new(&[]).err().unwrap();
        assert_eq!(
            error.to_string(),
            "Could not process plugin: Expected Wasm component, received unknown file type"
        );
        Ok(())
    }

    #[test]
    fn test_validate_plugin_with_module() -> Result<()> {
        let mut module = walrus::Module::with_config(ModuleConfig::default());
        let plugin_bytes = module.emit_wasm();
        let error = UninitializedPlugin::new(&plugin_bytes).err().unwrap();
        assert_eq!(
            error.to_string(),
            "Could not process plugin: Expected Wasm component, received Wasm module"
        );
        Ok(())
    }

    #[test]
    fn test_validate_plugin_with_everything_missing() -> Result<()> {
        let mut empty_module = walrus::Module::with_config(ModuleConfig::default());
        let plugin_bytes = encode_as_component(&empty_module.emit_wasm())?;
        let error = UninitializedPlugin::new(&plugin_bytes).err().unwrap();
        assert_eq!(
            error.to_string(),
            "Could not process plugin: No module with export named `invoke` found in component"
        );
        Ok(())
    }

    fn encode_as_component(module: &[u8]) -> Result<Vec<u8>> {
        ComponentEncoder::default().module(module)?.encode()
    }
}
