use anyhow::Result;
use quickjs_wasm_rs::ContextWrapper;

pub fn run_bytecode(context: &ContextWrapper, bytecode: &[u8]) -> Result<()> {
    context.eval_binary(bytecode)?;
    if cfg!(feature = "experimental_event_loop") {
        context.execute_pending()?;
    }
    Ok(())
}
