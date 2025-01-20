use crate::plugins::{InternalPlugin, Plugin, PluginKind};
use anyhow::Result;
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{Read, Seek},
    str,
};
use wasi_common::{pipe::WritePipe, sync::WasiCtxBuilder};
use wasmtime::{AsContextMut, Engine, Linker};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfigSchema {
    pub(crate) supported_properties: Vec<JsConfigProperty>,
}

impl ConfigSchema {
    pub(crate) fn from_plugin(plugin: &Plugin) -> Result<Option<ConfigSchema>> {
        if matches!(
            plugin.kind(),
            PluginKind::Internal(InternalPlugin::Legacy) | PluginKind::External
        ) {
            return Ok(None);
        }

        let engine = Engine::default();
        let module = wasmtime::Module::new(&engine, plugin.as_bytes())?;
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
        Ok(Some(Self {
            supported_properties: configs,
        }))
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
#[derive(Debug, Default)]
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
