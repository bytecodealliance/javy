use anyhow::{bail, Result};
use quickjs_wasm_rs::JSContextRef;

pub fn run_bytecode(context: &JSContextRef, bytecode: &[u8]) -> Result<()> {
    context.eval_binary(bytecode)?;
    if cfg!(feature = "experimental_event_loop") {
        context.execute_pending()?;
    } else if context.is_pending() {
        bail!("Adding tasks to the event queue is not supported");
    }
    Ok(())
}
