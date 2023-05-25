use anyhow::{bail, Result};
use javy::Runtime;

pub fn run_bytecode(runtime: &Runtime, bytecode: &[u8]) {
    run_bytecode_inner(runtime, bytecode).unwrap_or_else(|e| {
        eprintln!("Error while running JS: {}", e);
        std::process::abort();
    });
}

fn run_bytecode_inner(runtime: &Runtime, bytecode: &[u8]) -> Result<()> {
    let context = runtime.context();
    context.eval_binary(bytecode)?;
    if cfg!(feature = "experimental_event_loop") {
        context.execute_pending()?;
    } else if context.is_pending() {
        bail!("Adding tasks to the event queue is not supported");
    }
    Ok(())
}
