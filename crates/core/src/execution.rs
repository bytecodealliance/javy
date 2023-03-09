use std::env;

use anyhow::{bail, Result};
use quickjs_wasm_rs::Context;

pub fn run_bytecode(context: &Context, bytecode: &[u8]) -> Result<()> {
    context.eval_binary(bytecode)?;
    if cfg!(feature = "experimental_event_loop") {
        context.execute_pending()?;
    } else if context.is_pending() && !is_running_wpt_suite() {
        // WPT enqueues a promise as part of its suite setup and we can't change that behaviour so don't error
        bail!("Adding tasks to the event queue is not supported");
    }
    Ok(())
}

fn is_running_wpt_suite() -> bool {
    env::var("JAVY_WPT").is_ok()
}
