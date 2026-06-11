pub mod format;
mod interpreter;
mod state;

use state::{Profiler, State};
use std::cell::RefCell;
use std::io::Read;
use std::sync::OnceLock;

static STATE: OnceLock<State> = OnceLock::new();

thread_local! {
    /// Runtime profiling state.
    static PROFILER: RefCell<Profiler> = RefCell::new(Profiler::new());
}

fn state() -> &'static State {
    STATE
        .get()
        .expect("STATE must be initialized via `wizer.initialize`")
}

/// Run `f` with mutable access to the runtime [`Profiler`].
fn with_profiler<R>(f: impl FnOnce(&mut Profiler) -> R) -> R {
    PROFILER.with_borrow_mut(f)
}

/// Use Wizer to pre-initialize the user library module that will be
/// passed to Whamm. Note that this is temporary workaround until
/// Whamm grows such capability (see
/// https://github.com/ejrgilbert/whamm/issues/325).
#[unsafe(export_name = "wizer.initialize")]
pub extern "C" fn wizer_initialize() {
    let mut bytes = Vec::new();
    std::io::stdin()
        .read_to_end(&mut bytes)
        .expect("read target application bytes from stdin");
    let state = State::from_bytes(&bytes).expect("State initialization to work");
    STATE
        .set(state)
        .unwrap_or_else(|_| panic!("STATE must only be initialized once"));
}

/// Import namespace for the user library to be imported by the
/// monitor script.
pub static LIBRARY_NAME: &str = "profiler";

const MONITOR_TEMPLATE: &str = include_str!("monitor.mm");
const LIBRARY_NAME_PLACEHOLDER: &str = "{{LIBRARY_NAME}}";

/// Materialize the bundled whamm monitor script with [`LIBRARY_NAME`]
/// substituted.
pub fn monitor() -> Vec<u8> {
    MONITOR_TEMPLATE
        .replace(LIBRARY_NAME_PLACEHOLDER, LIBRARY_NAME)
        .into_bytes()
}

/// Returns true if the given index corresponds to the dispatch
/// function according to the heuristics defined in the [`state`]
/// module.
#[unsafe(no_mangle)]
pub extern "C" fn is_dispatch_func(index: u32) -> bool {
    state().is_dispatch_func(index)
}

/// Returns true if the given program counter offset is classified as
/// a dispatch load which feeds the index argument for the `br_table`
/// dispatch.
#[unsafe(no_mangle)]
pub extern "C" fn is_dispatch_load(fid: u32, pc: u32) -> bool {
    state().is_dispatch_load(fid, pc)
}

/// Start a new JS function frame.
#[unsafe(no_mangle)]
pub extern "C" fn start_func() {
    with_profiler(|p| p.start_func());
}

/// Pop the top most JS function frame. The outermost activation closes
/// out the final opcode with the instruction count at this point.
#[unsafe(no_mangle)]
pub extern "C" fn exit_func(instruction_count: i64) {
    with_profiler(|p| p.exit_func(instruction_count as u64));
}

/// Switch to the dispatch `br_table` target, closing out the previous
/// opcode with the instruction count at this point.
#[unsafe(no_mangle)]
pub extern "C" fn set_dispatch_target(target: u32, instruction_count: i64) {
    with_profiler(|p| p.set_dispatch_target(target, instruction_count as u64));
}

/// Set the effective address (i.e., the start address) of the current
/// function which uniquely identifies it.
#[unsafe(no_mangle)]
pub extern "C" fn set_func_addr(addr: u32) {
    with_profiler(|p| p.set_func_addr(addr));
}

/// Whether the opcode at offset `pc` in function `fid` is counted by
/// the profiler.
#[unsafe(no_mangle)]
pub extern "C" fn is_countable_opcode(fid: u32, pc: u32) -> bool {
    state().is_countable_opcode(fid, pc)
}

/// Flush the profiler results into a buffer.
#[unsafe(no_mangle)]
pub extern "C" fn report() {
    with_profiler(|p| p.report());
}

// TODO: Validate that `report` must run exactly once per profile
// invocation.
/// Linear-memory offset of the serialized report buffer.
#[unsafe(no_mangle)]
pub extern "C" fn report_ptr() -> u32 {
    with_profiler(|p| p.report_bytes().as_ptr() as u32)
}

/// Length in bytes of the serialized report buffer.
#[unsafe(no_mangle)]
pub extern "C" fn report_len() -> u32 {
    with_profiler(|p| p.report_bytes().len() as u32)
}

#[cfg(test)]
mod tests {
    use super::{LIBRARY_NAME, LIBRARY_NAME_PLACEHOLDER, monitor};
    use anyhow::Result;

    #[test]
    fn test_user_library_name() -> Result<()> {
        assert_eq!(LIBRARY_NAME, "profiler");
        Ok(())
    }

    #[test]
    fn monitor_substitutes_all_placeholders() -> Result<()> {
        let m = monitor();
        let s = std::str::from_utf8(&m)?;
        assert!(
            !s.contains(LIBRARY_NAME_PLACEHOLDER),
            "monitor() must replace every placeholder"
        );
        assert!(
            s.contains(LIBRARY_NAME),
            "monitor() must reference LIBRARY_NAME"
        );
        Ok(())
    }
}
