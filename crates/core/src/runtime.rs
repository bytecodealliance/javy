use anyhow::Result;
use javy::{Config, Runtime};

pub(crate) fn new_runtime() -> Result<Runtime> {
    Runtime::new(Config::default())
}
