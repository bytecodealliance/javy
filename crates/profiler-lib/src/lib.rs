mod interpreter;
mod state;

use state::State;
use std::io::Read;
use std::sync::OnceLock;

static STATE: OnceLock<State> = OnceLock::new();

fn state() -> &'static State {
    STATE
        .get()
        .expect("STATE must be initialized via `wizer.initialize`")
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
    todo!()
}

/// Exit the top most JS function frame.
#[unsafe(no_mangle)]
pub extern "C" fn exit_func() {
    todo!()
}

/// Set current dispatch function target. i.e., the `br_table` target.
#[unsafe(no_mangle)]
pub extern "C" fn set_dispatch_target(_target: u32) {
    todo!()
}

/// Set the effective address (i.e., the start address) of the current
/// function which uniquely identifies it.
#[unsafe(no_mangle)]
pub extern "C" fn set_func_addr(_addr: u32) {
    todo!()
}

/// Handle the execution of the given opcode.
#[unsafe(no_mangle)]
pub extern "C" fn handle_opcode(_pc: u32) {
    todo!()
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
