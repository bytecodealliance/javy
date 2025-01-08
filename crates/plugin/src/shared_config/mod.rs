//! APIs and data structures for receiving runtime configuration from the Javy CLI.

use anyhow::Result;
use javy_plugin_api::Config;
use serde::Deserialize;
use std::io::{stdout, Write};

mod runtime_config;

use crate::runtime_config;

runtime_config! {
    #[derive(Debug, Default, Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    pub struct SharedConfig {
        /// Whether to enable the `Javy.readSync` and `Javy.writeSync` builtins.
        javy_stream_io: Option<bool>,
        /// Whether to override the `JSON.parse` and `JSON.stringify`
        /// implementations with an alternative, more performant, SIMD based
        /// implemetation.
        simd_json_builtins: Option<bool>,
        /// Whether to enable support for the `TextEncoder` and `TextDecoder`
        /// APIs.
        text_encoding: Option<bool>,
        /// Whether to enable the event loop.
        event_loop: Option<bool>,
    }
}

impl SharedConfig {
    pub fn parse_from_json(config: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice::<Self>(config)?)
    }

    pub fn apply_to_config(&self, config: &mut Config) {
        if let Some(enable) = self.javy_stream_io {
            config.javy_stream_io(enable);
        }
        if let Some(enable) = self.simd_json_builtins {
            config.simd_json_builtins(enable);
        }
        if let Some(enable) = self.text_encoding {
            config.text_encoding(enable);
        }
        if let Some(enable) = self.event_loop {
            config.event_loop(enable);
        }
    }
}

#[export_name = "config_schema"]
pub fn config_schema() {
    stdout()
        .write_all(
            serde_json::to_string(&SharedConfig::config_schema())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
    stdout().flush().unwrap();
}
