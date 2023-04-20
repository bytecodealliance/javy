use anyhow::Result;
use javy::{Config, Runtime};

use crate::APIConfig;

pub trait RuntimeExt {
    fn new_with_apis(config: &Config, api_config: &APIConfig) -> Result<Runtime>;
}

impl RuntimeExt for Runtime {
    fn new_with_apis(config: &Config, api_config: &APIConfig) -> Result<Runtime> {
        let runtime = Runtime::new(config)?;
        crate::add_to_runtime(&runtime, api_config)?;
        Ok(runtime)
    }
}
