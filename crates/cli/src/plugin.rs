use anyhow::Result;
use javy_codegen::Plugin;

pub const PLUGIN_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.wasm"));
pub const QUICKJS_PROVIDER_V2_MODULE: &[u8] = include_bytes!("./javy_quickjs_provider_v2.wasm");

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
        Plugin::validate(bytes)?;
        Ok(Self { bytes })
    }

    /// Initializes the plugin.
    pub(crate) fn initialize(&self) -> Result<Vec<u8>> {
        javy_plugin_processing::initialize_plugin(self.bytes)
    }
}
