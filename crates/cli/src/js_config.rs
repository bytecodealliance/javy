use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{collections::HashMap, str};
use wasmtime::{AsContext, AsContextMut, Engine, Linker};

use crate::{CliPlugin, PluginKind};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfigSchema {
    pub(crate) supported_properties: Vec<JsConfigProperty>,
}

impl ConfigSchema {
    pub(crate) fn from_cli_plugin(cli_plugin: &CliPlugin) -> Result<Option<ConfigSchema>> {
        match cli_plugin.kind {
            PluginKind::User => Ok(None),
            PluginKind::Default => {
                let engine = Engine::default();
                let module = wasmtime::Module::new(&engine, cli_plugin.as_plugin().as_bytes())?;
                let mut linker = Linker::new(&engine);
                let mut store = wasmtime::Store::new(&engine, ());
                linker.define_unknown_imports_as_default_values(&mut store, &module)?;
                let instance = linker.instantiate(store.as_context_mut(), &module)?;

                let ret_area = instance
                    .get_typed_func::<(), i32>(store.as_context_mut(), "config-schema")?
                    .call(store.as_context_mut(), ())?;
                let memory = instance
                    .get_memory(store.as_context_mut(), "memory")
                    .ok_or_else(|| anyhow!("Missing memory export"))?;
                let mut buf = [0; 8];
                memory.read(store.as_context(), ret_area as usize, &mut buf)?;
                let offset = u32::from_le_bytes(buf[0..4].try_into().unwrap());
                let len = u32::from_le_bytes(buf[4..8].try_into().unwrap());
                let mut config_json = vec![0; len as usize];
                memory.read(store.as_context(), offset as usize, &mut config_json)?;

                let config_schema = serde_json::from_slice::<ConfigSchema>(&config_json)?;
                let mut configs = Vec::with_capacity(config_schema.supported_properties.len());
                for config in config_schema.supported_properties {
                    configs.push(JsConfigProperty {
                        name: config.name,
                        doc: config.doc,
                    });
                }

                Ok(Some(Self {
                    supported_properties: configs,
                }))
            }
        }
    }
}

/// A property that is in the config schema returned by the plugin.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JsConfigProperty {
    /// The name of the property (e.g., `simd-json-builtins`).
    pub(crate) name: String,
    /// The documentation to display for the property.
    pub(crate) doc: String,
}

/// A collection of property names to whether they are enabled.
#[derive(Clone, Debug, Default)]
pub(crate) struct JsConfig(HashMap<String, bool>);

impl JsConfig {
    /// Create from a hash.
    pub(crate) fn from_hash(configs: HashMap<String, bool>) -> Self {
        JsConfig(configs)
    }

    /// Encode as JSON.
    pub(crate) fn to_json(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    #[cfg(test)]
    /// Retrieve a value for a property name.
    pub(crate) fn get(&self, name: &str) -> Option<bool> {
        self.0.get(name).copied()
    }
}
