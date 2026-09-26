//! Instrumentation state to derive probe insertion.

use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use wirm::ir::module::module_functions::{FuncKind, LocalFunction};
use wirm::wasmparser::Operator;
use wirm::Module;

use crate::format;
use crate::interpreter;

/// Minimum number of *distinct* target blocks for a `br_table` to qualify as
/// the interpreter's opcode dispatch.
///
/// QuickJS contains several switches over roughly the opcode space, and most of
/// them also decode a byte out of a buffer, so neither table width nor
/// byte-load provenance singles out the interpreter.
pub const DISPATCH_DISTINCT_TARGETS: u32 = 100;

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
        Self::from_bytes_with_threshold(bytes, DISPATCH_DISTINCT_TARGETS)
    }

    /// Construct a `State` with a custom `br_table` target threshold.
    pub(crate) fn from_bytes_with_threshold(bytes: &[u8], threshold: u32) -> Result<Self> {
        let module = Module::parse(bytes, false, true)
            .map_err(|e| anyhow::anyhow!("failed to parse the target module: {e:?}"))?;

        let mut candidates: Vec<(u32, BTreeSet<u32>)> = vec![];
        for local in local_functions(&module) {
            if !has_dispatch_br_table(local, threshold) {
                continue;
            }
            let loads = interpreter::analyze(&module, local, threshold)?;
            if !loads.is_empty() {
                candidates.push((local.func_id.0, loads));
            }
        }

        let (dispatch_func_idx, dispatch_loads) = match candidates.as_slice() {
            [(idx, loads)] => (*idx, loads.clone()),
            [] => bail!(
                "No interpreter dispatch function found: no function contains a \
                 `br_table` with at least {threshold} distinct targets whose \
                 index is loaded from memory. Note that the dispatch loop lives \
                 in the Javy plugin, so a dynamically linked module cannot be \
                 instrumented; instrument the plugin instead."
            ),
            several => bail!(
                "Ambiguous interpreter dispatch function: {} functions contain a \
                 `br_table` with at least {threshold} distinct targets driven by \
                 a byte load (function indices {:?}). Exactly one is expected.",
                several.len(),
                several.iter().map(|(idx, _)| *idx).collect::<Vec<_>>()
            ),
        };

        let dispatch = local_function(&module, dispatch_func_idx)?;
        for &pc in &dispatch_loads {
            let op = operator_at(dispatch, pc)?;
            if !interpreter::is_byte_load(op) {
                bail!(
                    "internal error: pc {pc} of function {dispatch_func_idx} was \
                     recorded as a dispatch load but holds {op:?}"
                );
            }
        }

        let mut countable_opcodes = HashMap::new();
        for local in local_functions(&module) {
            let pcs = interpreter::countable_opcodes(local)?;
            if !pcs.is_empty() {
                countable_opcodes.insert(local.func_id.0, pcs);
            }
        }

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

/// Every local (non-imported) function in the module, in Wasm index order.
fn local_functions<'a, 'b>(module: &'a Module<'b>) -> impl Iterator<Item = &'a LocalFunction<'b>> {
    module
        .functions
        .iter()
        .filter_map(|func| match func.kind() {
            FuncKind::Local(local) => Some(&**local),
            FuncKind::Import(_) => None,
        })
}

/// The local function at Wasm function index `idx`.
fn local_function<'a, 'b>(module: &'a Module<'b>, idx: u32) -> Result<&'a LocalFunction<'b>> {
    local_functions(module)
        .find(|local| local.func_id.0 == idx)
        .ok_or_else(|| anyhow::anyhow!("no local function at index {idx}"))
}

/// The operator whamm binds `pc` to in `local`.
fn operator_at<'a, 'b>(local: &'a LocalFunction<'b>, pc: u32) -> Result<&'a Operator<'b>> {
    for (idx, op) in local.body.instructions.get_ops().iter().enumerate() {
        if interpreter::whamm_pc(local, idx)? == pc {
            return Ok(op);
        }
    }
    Err(anyhow::anyhow!(
        "no instruction at pc {pc} in function {}",
        local.func_id.0
    ))
}

/// The number of distinct target blocks a `br_table` reaches. Repeated targets
/// are cases sharing a handler, so they say nothing about how many opcodes the
/// switch really distinguishes.
fn distinct_targets(targets: &wirm::wasmparser::BrTable) -> Result<usize> {
    let depths: BTreeSet<u32> = targets.targets().collect::<Result<_, _>>()?;
    Ok(depths.len())
}

/// True iff `local` contains a `br_table` with at least `threshold` distinct
/// target blocks.
fn has_dispatch_br_table(local: &LocalFunction, threshold: u32) -> bool {
    local.body.instructions.get_ops().iter().any(|op| match op {
        Operator::BrTable { targets } => {
            distinct_targets(targets).is_ok_and(|n| n >= threshold as usize)
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};

    /// A wirm `Module` borrows the bytes it was parsed from, so tests hold the
    /// bytes and re-parse where they need to inspect the module.
    fn make(wat: &str, threshold: u32) -> Result<(Vec<u8>, State)> {
        let bytes = wat::parse_str(wat)?;
        let state = State::from_bytes_with_threshold(&bytes, threshold)?;
        Ok((bytes, state))
    }

    fn parse(bytes: &[u8]) -> Result<Module<'_>> {
        Module::parse(bytes, false, true).map_err(|e| anyhow!("failed to parse: {e:?}"))
    }

    /// Opening blocks for a dispatch with `n` distinct handlers. Each nested
    /// block is one branch target, so `br_table 0 1 .. n-1` reaches `n`
    /// distinct blocks — the shape of a real interpreter dispatch, where every
    /// opcode has its own handler.
    fn dispatch_open(n: usize) -> String {
        "(block ".repeat(n)
    }

    /// A `br_table` with `n` distinct targets. The trailing label is the
    /// default, which reuses the innermost block.
    fn br_table(n: usize) -> String {
        let labels = (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(" ");
        format!("br_table {labels} 0")
    }

    fn dispatch_close(n: usize) -> String {
        ")".repeat(n)
    }

    /// A module whose sole function dispatches on a byte through `n` distinct
    /// handlers, with `body` executed before the table.
    fn dispatch_module(n: usize, body: &str) -> String {
        format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32) (local $byte i32)
                {open}
                {body}
                {table}
                {close}))
            "#,
            open = dispatch_open(n),
            table = br_table(n),
            close = dispatch_close(n),
        )
    }

    /// Repeated targets are cases sharing a handler, so they do not count.
    #[test]
    fn distinct_targets_ignores_repeats() -> Result<()> {
        let shared = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32)
                {open}
                local.get $p
                i32.load8_u
                br_table 0 0 0 0 0
                {close}))
            "#,
            open = dispatch_open(3),
            close = dispatch_close(3),
        );
        let err = make(&shared, 3)
            .err()
            .expect("five cases sharing one block is not a dispatch");
        assert!(
            err.to_string().contains("No interpreter dispatch function"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    fn assert_all_byte_loads(bytes: &[u8], state: &State) -> Result<()> {
        let module = parse(bytes)?;
        let local = local_function(&module, state.dispatch_func_idx)?;
        for &pc in &state.dispatch_loads {
            let op = operator_at(local, pc)?;
            if !interpreter::is_byte_load(op) {
                bail!("pc {pc} is not a byte load: {op:?}");
            }
        }
        Ok(())
    }

    /// whamm derives `pc` as wirm's body-relative offset plus one, and neither
    /// half is documented — the bias lives in whamm's emitter, the base in
    /// wirm's parser. This pins the base against an independent walk and states
    /// the bias, so a change upstream shows up here rather than as a profiler
    /// that silently records nothing.
    #[test]
    fn whamm_pc_is_body_relative_plus_one() -> Result<()> {
        use wirm::wasmparser::{Parser, Payload};

        let bytes = wat::parse_str(
            r#"
            (module
              (memory 1)
              (func (param $p i32) (local $l i32)
                local.get $p
                drop))
            "#,
        )?;

        // Independently recover the coordinate from the raw binary.
        let mut expected = None;
        for payload in Parser::new(0).parse_all(&bytes) {
            if let Payload::CodeSectionEntry(body) = payload? {
                let locals_start = body
                    .get_locals_reader()?
                    .get_binary_reader()
                    .original_position();
                let (_, first_abs) = body
                    .get_operators_reader()?
                    .into_iter_with_offsets()
                    .next()
                    .ok_or_else(|| anyhow!("function has no instructions"))??;
                expected = Some((first_abs - locals_start + 1) as u32);
                break;
            }
        }
        let expected = expected.ok_or_else(|| anyhow!("no code section"))?;

        let module = parse(&bytes)?;
        let local = local_function(&module, 0)?;
        assert_eq!(
            interpreter::whamm_pc(local, 0)?,
            expected,
            "pc must be the body-relative offset plus one"
        );
        Ok(())
    }

    #[test]
    fn straight_line_load() -> Result<()> {
        let wat = dispatch_module(
            3,
            r#"
            local.get $p
            i32.load8_u
            "#,
        );
        let (bytes, state) = make(&wat, 3)?;

        assert!(state.is_dispatch_func(state.dispatch_func_idx));
        assert_eq!(state.dispatch_loads.len(), 1, "expected one dispatch load");
        assert_all_byte_loads(&bytes, &state)?;
        Ok(())
    }

    #[test]
    fn provenance_survives_i32_and() -> Result<()> {
        let wat = dispatch_module(
            3,
            r#"
            local.get $p
            i32.load8_u
            i32.const 0xff
            i32.and
            "#,
        );
        let (bytes, state) = make(&wat, 3)?;

        assert_eq!(
            state.dispatch_loads.len(),
            1,
            "and must not drop provenance"
        );
        assert_all_byte_loads(&bytes, &state)?;
        Ok(())
    }

    #[test]
    fn provenance_flows_through_local_roundtrip() -> Result<()> {
        let wat = dispatch_module(
            3,
            r#"
            local.get $p
            i32.load8_u
            local.set $byte
            local.get $byte
            "#,
        );
        let (bytes, state) = make(&wat, 3)?;

        assert_eq!(state.dispatch_loads.len(), 1);
        assert_all_byte_loads(&bytes, &state)?;
        Ok(())
    }

    #[test]
    fn if_else_merge_collects_both_loads() -> Result<()> {
        let wat = dispatch_module(
            3,
            r#"
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
            "#,
        );
        let (bytes, state) = make(&wat, 3)?;

        assert_eq!(state.dispatch_loads.len(), 2, "both loads must be recorded");
        assert_all_byte_loads(&bytes, &state)?;
        Ok(())
    }

    #[test]
    fn conditional_value() -> Result<()> {
        let wat = dispatch_module(
            3,
            r#"
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
            "#,
        );
        let (bytes, state) = make(&wat, 3)?;

        assert_eq!(state.dispatch_loads.len(), 2);
        assert_all_byte_loads(&bytes, &state)?;
        Ok(())
    }

    /// A dispatch-shaped `br_table` fed by a full-width load is not decoding
    /// opcodes, so a module containing only that has no dispatch function.
    #[test]
    fn non_byte_load_is_not_a_dispatch() -> Result<()> {
        let wat = dispatch_module(
            3,
            r#"
            local.get $p
            i32.load
            "#,
        );
        let err = make(&wat, 3)
            .err()
            .expect("a non-byte-driven br_table is not a dispatch");
        assert!(
            err.to_string().contains("No interpreter dispatch function"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    /// Likewise for a `br_table` on a constant: nothing was loaded, so nothing
    /// is being decoded.
    #[test]
    fn br_table_without_load_is_not_a_dispatch() -> Result<()> {
        let wat = dispatch_module(3, "i32.const 0");
        let err = make(&wat, 3)
            .err()
            .expect("a constant-index br_table is not a dispatch");
        assert!(
            err.to_string().contains("No interpreter dispatch function"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    /// Two byte-driven dispatches in one module is not something a Javy module
    /// produces, and guessing between them would silently profile the wrong
    /// function, so it is an error naming both.
    #[test]
    fn two_byte_driven_dispatches_are_ambiguous() -> Result<()> {
        let func = format!(
            r#"
            (func (param $p i32)
              {open}
              local.get $p
              i32.load8_u
              {table}
              {close})
            "#,
            open = dispatch_open(3),
            table = br_table(3),
            close = dispatch_close(3),
        );
        let wat = format!("(module (memory 1) {func} {func})");
        let err = make(&wat, 3)
            .err()
            .expect("two dispatches must be an error");
        let msg = err.to_string();
        assert!(
            msg.contains("Ambiguous interpreter dispatch function"),
            "unexpected error: {msg}"
        );
        Ok(())
    }

    /// Width alone would pick the wrong function. Measured on a Javy module: a
    /// 254-target switch collapsing onto 4 blocks sits alongside the
    /// interpreter's 251-target, 225-block dispatch.
    #[test]
    fn wide_br_table_without_byte_load_provenance_is_not_a_candidate() -> Result<()> {
        let wat = format!(
            r#"
            (module
              (memory 1)
              (func (param $p i32)
                {open_wide}
                  local.get $p
                  i32.load
                  {wide}
                {close_wide})
              (func (param $p i32)
                {open_narrow}
                  local.get $p
                  i32.load8_u
                  {narrow}
                {close_narrow}))
            "#,
            open_wide = dispatch_open(6),
            close_wide = dispatch_close(6),
            open_narrow = dispatch_open(3),
            close_narrow = dispatch_close(3),
            // The impostor: a wider table, but its index is a full i32 load.
            wide = br_table(6),
            // The interpreter: byte-driven, and still over the threshold.
            narrow = br_table(3)
        );
        let (_module, state) = make(&wat, 3)?;

        assert_eq!(
            state.dispatch_func_idx, 1,
            "the byte-driven dispatcher must win over the wider non-byte one"
        );
        Ok(())
    }

    #[test]
    fn no_candidate_is_an_error_that_mentions_dynamic_linking() -> Result<()> {
        let wat = r#"
            (module
              (memory 1)
              (func (param $p i32)
                local.get $p
                drop))
        "#;
        let err = make(wat, 3)
            .err()
            .expect("missing dispatch must be an error");
        let msg = err.to_string();
        assert!(
            msg.contains("No interpreter dispatch function found"),
            "unexpected error: {msg}"
        );
        assert!(
            msg.contains("dynamically linked"),
            "the error should point at the dynamic-linking case: {msg}"
        );
        Ok(())
    }

    #[test]
    fn correctly_identifies_dispatch_func() -> Result<()> {
        let wat = dispatch_module(
            3,
            r#"
            local.get $p
            i32.load8_u
            "#,
        );
        let (_module, state) = make(&wat, 3)?;

        assert!(state.is_dispatch_func(state.dispatch_func_idx));
        assert!(!state.is_dispatch_func(state.dispatch_func_idx + 1));
        Ok(())
    }

    #[test]
    fn dispatch_load_is_scoped_to_dispatch_func() -> Result<()> {
        let wat = dispatch_module(
            3,
            r#"
            local.get $p
            i32.load8_u
            "#,
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
