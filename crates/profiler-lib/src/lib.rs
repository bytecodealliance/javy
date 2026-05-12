mod ai;
mod state;

use state::State;
use std::sync::LazyLock;

// TODO: Passing empty bytes is temporary. Whamm currently does not
//       offer a mechanism to pass in the target application bytes.
//       Prior to hooking the bytes we need to either find the best
//       way to accomplish that in Whamm's instrumentation pass or
//       create a custom pass in Javy, e.g., through Wizer. Ideally we
//       want the former: arguably, having access to the bytes is
//       something that other libraries might need.
static STATE: std::sync::LazyLock<State> =
    LazyLock::new(|| State::from_bytes(&[]).expect("State initialization to work"));

/// Returns true if the given index corresponds to the dispatch
/// function according to the heuristics defined in the [`state`]
/// module.
#[unsafe(no_mangle)]
pub extern "C" fn is_dipatch_func(index: u32) -> bool {
    STATE.is_dispatch_func(index)
}

/// Returns true if the given program counter offset is classified as
/// a dispatch load which feeds the index argument for the `br_table`
/// dispatch.
#[unsafe(no_mangle)]
pub extern "C" fn is_dispatch_load(fid: u32, pc: u32) -> bool {
    STATE.is_dispatch_load(fid, pc)
}
