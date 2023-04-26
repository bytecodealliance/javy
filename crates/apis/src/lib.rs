use anyhow::Result;
use api_config::APIConfig;
use javy::Runtime;

pub use runtime_ext::RuntimeExt;

mod api_config;
mod runtime_ext;

pub(crate) trait JSApiSet {
    fn register(&self, runtime: &Runtime, config: APIConfig) -> Result<()>;
}

pub fn add_to_runtime(_runtime: &Runtime, _config: APIConfig) -> Result<()> {
    Ok(())
}
