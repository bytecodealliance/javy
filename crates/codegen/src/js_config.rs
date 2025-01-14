use anyhow::Result;
use std::collections::HashMap;

/// A collection of property names to whether they are enabled.
#[derive(Debug, Default)]
pub struct JsConfig(HashMap<String, bool>);

impl JsConfig {
    /// Create from a hash.
    pub fn from_hash(configs: HashMap<String, bool>) -> Self {
        JsConfig(configs)
    }

    /// Encode as JSON.
    pub fn to_json(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    /// Retrieve a value for a property name.
    pub fn get(&self, name: &str) -> Option<bool> {
        self.0.get(name).copied()
    }
}
