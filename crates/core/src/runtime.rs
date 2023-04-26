use anyhow::Result;
use javy::Runtime;
use javy_apis::RuntimeExt;

pub(crate) fn new_runtime() -> Result<Runtime> {
    Runtime::new_with_defaults()
}
