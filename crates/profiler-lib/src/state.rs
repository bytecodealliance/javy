//! Instrumentation state to derive probe insertion.

use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use walrus::ir::{BrTable, Visitor, dfs_in_order};
use walrus::{FunctionId, FunctionKind, LocalFunction, ModuleConfig};

use crate::format;
use crate::interpreter;

/// Threshold above which a `br_table` qualifies as the interpreter
/// dispatch loop.
pub const DISPATCH_TARGET_THRESHOLD: u32 = 250;

pub struct State {
    /// Wasm function index, which contains a `br_table` with at least
    /// the configured target threshold. There should be a single
    /// function which meets this criteria.
    pub dispatch_func_idx: u32,
    /// Byte offsets of the `i32.load8_u` instructions in the dispatch
    /// function whose values feed the dispatch `br_table`'s index.
    dispatch_loads: BTreeSet<u32>,
    /// Per function, the byte offsets of the opcodes the
    /// profiler counts.
    /// Calculated eagerly so that at runtime the calculation becomes
    /// a simple lookup in the `BTreeSet`.
    countable_opcodes: HashMap<u32, BTreeSet<u32>>,
}

impl State {
    /// Construct a `State` from the given Wasm bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_bytes_with_threshold(bytes, DISPATCH_TARGET_THRESHOLD)
    }

    /// Construct a `State` with a custom `br_table` target threshold.
    pub(crate) fn from_bytes_with_threshold(bytes: &[u8], threshold: u32) -> Result<Self> {
        let module = ModuleConfig::new().parse(bytes)?;

        let candidates: Vec<(u32, FunctionId)> = module
            .funcs
            .iter()
            .enumerate()
            .filter_map(|(idx, func)| match &func.kind {
                FunctionKind::Local(local) if has_large_br_table(local, threshold) => {
                    Some((idx as u32, func.id()))
                }
                _ => None,
            })
            .collect();

        if candidates.len() != 1 {
            bail!(
                "Unexpected number of dispatch functions. Expected 1, found {}",
                candidates.len()
            );
        }

        let (dispatch_func_idx, dispatch_func_id) = candidates[0];

        let local = match &module.funcs.get(dispatch_func_id).kind {
            FunctionKind::Local(l) => l,
            // Mostly for completeness, this should not be possible,
            // given the filtering above.
            _ => unreachable!("filtered to local functions only"),
        };
        let dispatch_loads = interpreter::analyze(&module, local, threshold);

        let countable_opcodes = module
            .funcs
            .iter()
            .enumerate()
            .filter_map(|(idx, func)| match &func.kind {
                FunctionKind::Local(local) => {
                    let pcs = interpreter::countable_opcodes(local);
                    (!pcs.is_empty()).then_some((idx as u32, pcs))
                }
                _ => None,
            })
            .collect();

        Ok(Self {
            dispatch_func_idx,
            dispatch_loads,
            countable_opcodes,
        })
    }

    /// Given a function id in the module, return whether it matches
    /// the dispatch function heuristics.
    pub fn is_dispatch_func(&self, id: u32) -> bool {
        self.dispatch_func_idx == id
    }

    /// Given a function id and an instruction offset, return whether
    /// the instruction at `pc` is the memory load responsible for
    /// fetching the next QuickJS opcode.
    pub fn is_dispatch_load(&self, id: u32, pc: u32) -> bool {
        id == self.dispatch_func_idx && self.dispatch_loads.contains(&pc)
    }

    /// Whether the opcode at offset `pc` in function `fid` is one the
    /// profiler cares about.
    pub fn is_countable_opcode(&self, fid: u32, pc: u32) -> bool {
        self.countable_opcodes
            .get(&fid)
            .is_some_and(|pcs| pcs.contains(&pc))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct DispatchTarget(u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FuncAddr(u32);

/// A JavaScript function frame.
#[derive(Default)]
struct Frame {
    /// The function's bytecode buffer start address, which uniquely
    /// identifies the JS function being executed.
    func_addr: Option<FuncAddr>,
}

/// Profiling state.
///
/// It tracks the interpreter's stack frames and accumulates the cost
/// of each JS opcode, per JS function.
///
/// Cost is an approximation: an opcode is charged every instruction
/// from the dispatch into its handler until the next dispatch. That spans
/// the handler, any helper functions it calls, and the decode of the
/// following opcode. This could result in an overapproximation in
/// some cases, e.g., handling the last opcode in the bytecode buffer
/// for a given JS function.
#[derive(Default)]
pub struct Profiler {
    /// JS function frames.
    stack: Vec<Frame>,
    /// The current dispatch target scope.
    current: Option<(FuncAddr, DispatchTarget)>,
    /// Instruction count belonging to the last dispatch target.
    last_instruction_count: u64,
    /// Per-function-and-dispatch target count.
    counts: BTreeMap<(FuncAddr, DispatchTarget), u64>,
    /// Serialized counts report.
    report: Vec<u8>,
}

impl Profiler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new interpreter frame.
    pub fn start_func(&mut self) {
        self.stack.push(Frame::default());
    }

    /// End the current function's frame. When the outermost interpreter
    /// activation returns, close out the final opcode passing in the
    /// instruction count up until that point.
    pub fn exit_func(&mut self, instruction_count: u64) {
        self.stack.pop().expect("Function frame to exist");
        if self.stack.is_empty() {
            if let Some(key) = self.current {
                *self.counts.entry(key).or_default() +=
                    instruction_count.saturating_sub(self.last_instruction_count);
            }
            self.last_instruction_count = instruction_count;
            self.current = None;
        }
    }

    /// Switch the dispatch target to `target` (the QuickJS opcode whose
    /// handler is about to run). This also closes out the opcode that was
    /// running, charging it the instructions executed since the previous
    /// switch. `instruction_count` is the running count of executed
    /// countable Wasm instructions.
    pub fn set_dispatch_target(&mut self, target: u32, instruction_count: u64) {
        // Close out the opcode that just finished.
        if let Some(key) = self.current {
            *self.counts.entry(key).or_default() +=
                instruction_count.saturating_sub(self.last_instruction_count);
        }
        self.last_instruction_count = instruction_count;

        // Begin attributing to the opcode being dispatched.
        if let Some(addr) = self.stack.last().and_then(|frame| frame.func_addr) {
            self.current = Some((addr, DispatchTarget(target)));
        }
    }

    /// Record the start address of the topmost function frame.
    pub fn set_func_addr(&mut self, addr: u32) {
        if let Some(frame) = self.stack.last_mut() {
            frame.func_addr = Some(FuncAddr(addr));
        }
    }

    /// Serialize the accumulated counts into the internal report buffer
    /// using the [`crate::format`] encoding. Call once at program exit;
    /// afterwards [`Profiler::report_bytes`] exposes the buffer for the
    /// host to read out of linear memory.
    pub fn report(&mut self) {
        self.report =
            format::write(
                self.counts
                    .iter()
                    .map(|(&(addr, target), &count)| format::Record {
                        func_addr: addr.0,
                        target: target.0,
                        count,
                    }),
            );
    }

    /// The serialized report buffer.
    pub fn report_bytes(&self) -> &[u8] {
        &self.report
    }
}

/// True iff `func` contains a `br_table` whose target list has at
/// least `threshold` entries.
fn has_large_br_table(func: &LocalFunction, threshold: u32) -> bool {
    struct Detect {
        threshold: u32,
        found: bool,
    }
    impl<'instr> Visitor<'instr> for Detect {
        fn visit_br_table(&mut self, br_table: &BrTable) {
            if br_table.blocks.len() >= self.threshold as usize {
                self.found = true;
            }
        }
    }
    let mut d = Detect {
        threshold,
        found: false,
    };
    dfs_in_order(&mut d, func, func.entry_block());
    d.found
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, anyhow};
    use std::collections::HashMap;
    use walrus::InstrLocId;
    use walrus::Module;
    use walrus::ir::Instr;

    fn make(wat: &str, threshold: u32) -> Result<(Module, State)> {
        let bytes = wat::parse_str(wat)?;
        let module = ModuleConfig::new().parse(&bytes)?;
        let state = State::from_bytes_with_threshold(&bytes, threshold)?;
        Ok((module, state))
    }

    /// Map every instruction's byte offset to the corresponding
    /// instruction in the function at index `fid`.
    fn pc2instr(module: &Module, fid: u32) -> Result<HashMap<u32, Instr>> {
        let func = module
            .funcs
            .iter()
            .nth(fid as usize)
            .ok_or_else(|| anyhow!("no function at index {fid}"))?;
        let local = match &func.kind {
            FunctionKind::Local(l) => l,
            _ => bail!("function at index {fid} is not a local function"),
        };

        #[derive(Default)]
        struct Collect {
            map: HashMap<u32, Instr>,
        }
        impl<'i> Visitor<'i> for Collect {
            fn visit_instr(&mut self, instr: &'i Instr, loc: &'i InstrLocId) {
                self.map.insert(loc.data(), instr.clone());
            }
        }
        let mut v = Collect::default();
        dfs_in_order(&mut v, local, local.entry_block());
        Ok(v.map)
    }

    fn br_table(n: usize) -> String {
        let labels = vec!["0"; n].join(" ");
        format!("br_table {labels} 0")
    }

    fn assert_all_byte_loads(module: &Module, state: &State) -> Result<()> {
        let map = pc2instr(module, state.dispatch_func_idx)?;
        for &pc in &state.dispatch_loads {
            let instr = map
                .get(&pc)
                .ok_or_else(|| anyhow!("pc {pc} not found in function"))?;
            match instr {
                Instr::Load(l) => {
                    if !interpreter::is_byte_load(l) {
                        bail!("pc {pc} is a load but not a byte load: {:?}", l.kind);
                    }
                }
                other => bail!("pc {pc} is {other:?}, not a load"),
            }
        }
        Ok(())
    }

    #[test]
    fn straight_line_load() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32)
                (block
                  local.get $p
                  i32.load8_u
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (module, state) = make(&wat, 3)?;

        assert!(state.is_dispatch_func(state.dispatch_func_idx));
        assert_eq!(state.dispatch_loads.len(), 1, "expected one dispatch load");
        assert_all_byte_loads(&module, &state)?;
        Ok(())
    }

    #[test]
    fn provenance_survives_i32_and() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32)
                (block
                  local.get $p
                  i32.load8_u
                  i32.const 0xff
                  i32.and
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (module, state) = make(&wat, 3)?;

        assert_eq!(
            state.dispatch_loads.len(),
            1,
            "and must not drop provenance"
        );
        assert_all_byte_loads(&module, &state)?;
        Ok(())
    }

    #[test]
    fn provenance_flows_through_local_roundtrip() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32) (local $byte i32)
                (block
                  local.get $p
                  i32.load8_u
                  local.set $byte
                  local.get $byte
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (module, state) = make(&wat, 3)?;

        assert_eq!(state.dispatch_loads.len(), 1);
        assert_all_byte_loads(&module, &state)?;
        Ok(())
    }

    #[test]
    fn if_else_merge_collects_both_loads() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32) (local $byte i32)
                (block
                  local.get $p
                  i32.const 1
                  i32.lt_s
                  if
                    local.get $p
                    i32.load8_u offset=0
                    local.set $byte
                  else
                    local.get $p
                    i32.load8_u offset=4
                    local.set $byte
                  end
                  local.get $byte
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (module, state) = make(&wat, 3)?;

        assert_eq!(state.dispatch_loads.len(), 2, "both loads must be recorded");
        assert_all_byte_loads(&module, &state)?;
        Ok(())
    }

    #[test]
    fn conditional_value() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32) (local $byte i32)
                (block
                  local.get $p
                  i32.load8_u offset=0
                  local.set $byte
                  local.get $p
                  i32.const 200
                  i32.lt_s
                  if
                    local.get $p
                    i32.load8_u offset=1
                    local.set $byte
                  end
                  local.get $byte
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (module, state) = make(&wat, 3)?;

        assert_eq!(state.dispatch_loads.len(), 2);
        assert_all_byte_loads(&module, &state)?;
        Ok(())
    }

    #[test]
    fn non_byte_load_is_not_recorded() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32)
                (block
                  local.get $p
                  i32.load
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (_module, state) = make(&wat, 3)?;

        assert!(
            state.dispatch_loads.is_empty(),
            "non-byte load must not be recorded"
        );
        Ok(())
    }

    #[test]
    fn br_table_without_load_is_empty() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (func (param $p i32)
                (block
                  i32.const 0
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (_module, state) = make(&wat, 3)?;

        assert!(state.dispatch_loads.is_empty());
        Ok(())
    }

    #[test]
    fn correctly_identifies_dispatch_func() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32)
                (block
                  local.get $p
                  i32.load8_u
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (_module, state) = make(&wat, 3)?;

        assert!(state.is_dispatch_func(state.dispatch_func_idx));
        assert!(!state.is_dispatch_func(state.dispatch_func_idx + 1));
        Ok(())
    }

    #[test]
    fn dispatch_load_is_scoped_to_dispatch_func() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32)
                (block
                  local.get $p
                  i32.load8_u
                  {br_table})))
            "#,
            br_table = br_table(3)
        );
        let (_module, state) = make(&wat, 3)?;

        let pc = *state.dispatch_loads.iter().next().unwrap();
        assert!(state.is_dispatch_load(state.dispatch_func_idx, pc));
        assert!(!state.is_dispatch_load(state.dispatch_func_idx + 1, pc));
        Ok(())
    }

    #[test]
    fn set_dispatch_target_attributes_deltas_per_opcode() {
        let mut p = Profiler::new();
        p.start_func();
        p.set_func_addr(0x1000);

        p.set_dispatch_target(5, 0);
        p.set_dispatch_target(7, 3);
        p.set_dispatch_target(5, 8);
        p.exit_func(12);

        let f = FuncAddr(0x1000);
        assert_eq!(p.counts.get(&(f, DispatchTarget(5))), Some(&7)); // 3 + 4
        assert_eq!(p.counts.get(&(f, DispatchTarget(7))), Some(&5));
        assert_eq!(p.counts.len(), 2);
    }

    #[test]
    fn first_dispatch_target_charges_nothing() {
        let mut p = Profiler::new();
        p.start_func();
        p.set_func_addr(0x10);
        p.set_dispatch_target(0, 5);
        assert!(p.counts.is_empty());

        p.set_dispatch_target(1, 8);
        assert_eq!(p.counts.get(&(FuncAddr(0x10), DispatchTarget(0))), Some(&3));
    }

    #[test]
    fn exit_func_restores_caller_function() {
        let mut p = Profiler::new();
        p.start_func();
        p.set_func_addr(0xA00);
        p.set_dispatch_target(1, 0);

        // Nested call.
        p.start_func();
        p.set_func_addr(0xB00);
        p.set_dispatch_target(2, 10);
        p.exit_func(13);

        // Back to the parent function call.
        p.set_dispatch_target(3, 15);
        p.exit_func(20);

        assert_eq!(
            p.counts.get(&(FuncAddr(0xA00), DispatchTarget(1))),
            Some(&10)
        );
        assert_eq!(
            p.counts.get(&(FuncAddr(0xB00), DispatchTarget(2))),
            Some(&5)
        );
        assert_eq!(
            p.counts.get(&(FuncAddr(0xA00), DispatchTarget(3))),
            Some(&5)
        );
    }

    #[test]
    fn report_encodes_records_in_order() {
        let mut p = Profiler::new();
        p.start_func();
        p.set_func_addr(0x1000);
        p.set_dispatch_target(7, 0);
        p.set_dispatch_target(5, 5);
        p.exit_func(8);

        p.report();
        let records = format::read(p.report_bytes()).unwrap();

        assert_eq!(
            records,
            vec![
                format::Record {
                    func_addr: 0x1000,
                    target: 5,
                    count: 3
                },
                format::Record {
                    func_addr: 0x1000,
                    target: 7,
                    count: 5
                },
            ]
        );
    }

    #[test]
    fn report_with_no_counts_reads_back_empty() {
        let mut p = Profiler::new();
        p.report();
        assert!(format::read(p.report_bytes()).unwrap().is_empty());
    }
}
