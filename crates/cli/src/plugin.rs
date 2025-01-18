use crate::codegen::plugin::Plugin;

pub const PLUGIN_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.wasm"));
pub const QUICKJS_PROVIDER_V2_MODULE: &[u8] = include_bytes!("./javy_quickjs_provider_v2.wasm");

pub(crate) enum PluginKind {
    None,
    Default,
}

pub(crate) struct CliPlugin {
    pub(crate) plugin: Plugin,
    pub(crate) kind: PluginKind,
}

impl CliPlugin {
    pub fn new(plugin: Plugin, kind: PluginKind) -> Self {
        CliPlugin {
            plugin: plugin,
            kind: kind,
        }
    }

    pub fn as_plugin(&self) -> &Plugin {
        &self.plugin
    }

    pub fn into_plugin(self) -> Plugin {
        self.plugin
    }
}
