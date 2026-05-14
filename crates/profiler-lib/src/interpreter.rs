//! Abstract interpretation for `br_target` provenance.
//!
//! Given a WebAssembly module and a local function id, perform
//! abstract interpetation to determine the byte offset provenance of
//! the br_table index argument.
//! The target br_table instruction is chosen using a fixed number of
//! branch targets as heuristic.

use std::collections::{BTreeSet, HashMap};
use walrus::ir::{
    Binop, Block, Br, BrIf, BrTable, Call, CallIndirect, Const, Drop, GlobalGet, GlobalSet, IfElse,
    Instr, InstrSeq, InstrSeqId, Load, LoadKind, LocalGet, LocalSet, LocalTee, Loop, MemoryGrow,
    MemorySize, Return, ReturnCall, ReturnCallIndirect, Select, Store, Unop, Unreachable, Visitor,
    dfs_in_order,
};
use walrus::{InstrLocId, LocalFunction, LocalId, Module};

/// The set of offsets of byte-load instructions that contribute to
/// the br_table index argument.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Provenance(BTreeSet<u32>);

impl Provenance {
    fn new() -> Self {
        Self(BTreeSet::new())
    }

    fn with(pos: u32) -> Self {
        let mut s = BTreeSet::new();
        s.insert(pos);
        Self(s)
    }

    fn join_in_place(&mut self, other: &Provenance) {
        for &v in &other.0 {
            self.0.insert(v);
        }
    }

    fn joined(&self, other: &Provenance) -> Provenance {
        let mut out = self.clone();
        out.join_in_place(other);
        out
    }
}

#[derive(Clone, Debug, Default)]
struct AbstractState {
    stack: Vec<Provenance>,
    locals: HashMap<LocalId, Provenance>,
}

impl AbstractState {
    fn join(&mut self, other: &AbstractState) {
        // Wasm spec ensures that stack length must match at merge
        // points.
        let n = self.stack.len().min(other.stack.len());
        for i in 0..n {
            self.stack[i].join_in_place(&other.stack[i]);
        }
        for (k, v) in &other.locals {
            self.locals
                .entry(*k)
                .and_modify(|cur| cur.join_in_place(v))
                .or_insert_with(|| v.clone());
        }
    }

    fn pop(&mut self) -> Provenance {
        self.stack.pop().unwrap_or_default()
    }

    fn push(&mut self, p: Provenance) {
        self.stack.push(p);
    }
}

/// Control frames.
enum ControlFrame {
    Block {
        seq_id: InstrSeqId,
        target: AbstractState,
    },
    Loop {
        seq_id: InstrSeqId,
        header: AbstractState,
    },
    If {
        seq_id: InstrSeqId,
        entry_state: AbstractState,
        target: AbstractState,
    },
    Else {
        seq_id: InstrSeqId,
        if_exit: AbstractState,
        target: AbstractState,
    },
}

impl ControlFrame {
    fn seq_id(&self) -> InstrSeqId {
        match self {
            Self::Block { seq_id, .. }
            | Self::Loop { seq_id, .. }
            | Self::If { seq_id, .. }
            | Self::Else { seq_id, .. } => *seq_id,
        }
    }
}

/// Analyze the function with a custom `br_table` threshold.
pub(crate) fn analyze(module: &Module, func: &LocalFunction, threshold: u32) -> BTreeSet<u32> {
    let mut interp = AbstractInterp::new(module, func, threshold);
    dfs_in_order(&mut interp, func, func.entry_block());
    interp.dispatch_loads
}

struct AbstractInterp<'a> {
    /// The target module.
    module: &'a Module,
    /// Interpreter state.
    state: AbstractState,
    /// Control frames.
    frames: Vec<ControlFrame>,
    /// Program counter (byte offset of the instruction being visited).
    pc: u32,
    /// The set of loads that contribute the index argument to the
    /// `br_table`.
    dispatch_loads: BTreeSet<u32>,
    /// Threshold above which a `br_table` qualifies as the
    /// interpreter dispatch loop.
    dispatch_target_threshold: u32,
}

impl<'a> AbstractInterp<'a> {
    fn new(module: &'a Module, func: &'a LocalFunction, threshold: u32) -> Self {
        let mut state = AbstractState::default();
        for arg in &func.args {
            state.locals.insert(*arg, Provenance::new());
        }

        let frames = vec![
            // Push the implicit start control block.
            ControlFrame::Block {
                seq_id: func.entry_block(),
                target: AbstractState::default(),
            },
        ];

        Self {
            module,
            state,
            frames,
            pc: 0,
            dispatch_target_threshold: threshold,
            dispatch_loads: BTreeSet::new(),
        }
    }

    /// Blanket pop/push for operators deemed not to affect
    /// provenance.
    fn pop_push_n(&mut self, n_pop: usize, n_push: usize) {
        for _ in 0..n_pop {
            self.state.pop();
        }
        for _ in 0..n_push {
            self.state.push(Provenance::new());
        }
    }

    /// Merge the current state into the target branch state.
    fn join_into_target(&mut self, target: InstrSeqId) {
        let snapshot = self.state.clone();
        if let Some(frame) = self.frames.iter_mut().rev().find(|f| f.seq_id() == target) {
            match frame {
                ControlFrame::Loop { header, .. } => header.join(&snapshot),
                ControlFrame::Block { target, .. }
                | ControlFrame::If { target, .. }
                | ControlFrame::Else { target, .. } => target.join(&snapshot),
            }
        }
    }
}

pub fn is_byte_load(load: &Load) -> bool {
    matches!(load.kind, LoadKind::I32_8 { .. } | LoadKind::I64_8 { .. })
}

impl<'f, 'instr> Visitor<'instr> for AbstractInterp<'f> {
    fn visit_instr(&mut self, _: &'instr Instr, loc: &'instr InstrLocId) {
        // Save the program counter before visiting each operator.
        self.pc = loc.data();
    }

    fn end_instr_seq(&mut self, _: &'instr InstrSeq) {
        let frame = match self.frames.pop() {
            Some(f) => f,
            None => return,
        };
        match frame {
            // On block end, join the state of the ending block into
            // the current state.
            ControlFrame::Block { target, .. } => {
                self.state.join(&target);
            }
            // On loop end, we have a fall-through.
            ControlFrame::Loop { .. } => {}
            // On if end, replace the frame with else, and restore the
            // state to the if entry state.
            // Also, merge and store the exit state by merging the
            // current state with the target state.
            ControlFrame::If {
                entry_state,
                target,
                ..
            } => {
                let mut exit_state = self.state.clone();
                exit_state.join(&target);
                match self.frames.last_mut() {
                    Some(ControlFrame::Else { if_exit: slot, .. }) => *slot = exit_state,
                    _ => panic!("If frame must be followed by matching Else frame on the stack"),
                }
                self.state = entry_state;
            }
            // On else end, merge the current state with the state
            // from both the if and else.
            ControlFrame::Else {
                if_exit, target, ..
            } => {
                self.state.join(&target);
                self.state.join(&if_exit);
            }
        }
    }

    fn visit_block(&mut self, b: &Block) {
        self.frames.push(ControlFrame::Block {
            seq_id: b.seq,
            target: AbstractState::default(),
        });
    }

    fn visit_loop(&mut self, l: &Loop) {
        self.frames.push(ControlFrame::Loop {
            seq_id: l.seq,
            header: self.state.clone(),
        });
    }

    fn visit_if_else(&mut self, ie: &IfElse) {
        self.state.pop();
        let snapshot = self.state.clone();

        self.frames.push(ControlFrame::Else {
            seq_id: ie.alternative,
            if_exit: AbstractState::default(),
            target: AbstractState::default(),
        });
        self.frames.push(ControlFrame::If {
            seq_id: ie.consequent,
            entry_state: snapshot,
            target: AbstractState::default(),
        });
    }

    fn visit_br(&mut self, br: &Br) {
        self.join_into_target(br.block);
    }

    fn visit_br_if(&mut self, br_if: &BrIf) {
        self.state.pop();
        self.join_into_target(br_if.block);
    }

    fn visit_br_table(&mut self, bt: &BrTable) {
        let index = self.state.pop();
        if bt.blocks.len() >= self.dispatch_target_threshold as usize {
            self.dispatch_loads.extend(index.0.iter().copied());
        }
        let snapshot = self.state.clone();
        let mut seen: BTreeSet<InstrSeqId> = BTreeSet::new();
        for &t in bt.blocks.iter().chain(std::iter::once(&bt.default)) {
            if !seen.insert(t) {
                continue;
            }
            if let Some(frame) = self.frames.iter_mut().rev().find(|f| f.seq_id() == t) {
                match frame {
                    ControlFrame::Loop { header, .. } => header.join(&snapshot),
                    ControlFrame::Block { target, .. }
                    | ControlFrame::If { target, .. }
                    | ControlFrame::Else { target, .. } => target.join(&snapshot),
                }
            }
        }
    }

    fn visit_load(&mut self, l: &Load) {
        self.state.pop();
        if is_byte_load(l) {
            self.state.push(Provenance::with(self.pc));
        } else {
            self.state.push(Provenance::new());
        }
    }

    fn visit_store(&mut self, _: &Store) {
        self.pop_push_n(2, 0);
    }

    fn visit_const(&mut self, _: &Const) {
        self.state.push(Provenance::new());
    }

    fn visit_binop(&mut self, _: &Binop) {
        let rhs = self.state.pop();
        let lhs = self.state.pop();
        self.state.push(lhs.joined(&rhs));
    }

    fn visit_unop(&mut self, _: &Unop) {
        let v = self.state.pop();
        self.state.push(v);
    }

    fn visit_local_get(&mut self, lg: &LocalGet) {
        let p = self
            .state
            .locals
            .get(&lg.local)
            .cloned()
            .unwrap_or_default();
        self.state.push(p);
    }

    fn visit_local_set(&mut self, ls: &LocalSet) {
        let v = self.state.pop();
        self.state.locals.insert(ls.local, v);
    }

    fn visit_local_tee(&mut self, lt: &LocalTee) {
        let v = self.state.stack.last().cloned().unwrap_or_default();
        self.state.locals.insert(lt.local, v);
    }

    fn visit_global_get(&mut self, _: &GlobalGet) {
        self.state.push(Provenance::new());
    }

    fn visit_global_set(&mut self, _: &GlobalSet) {
        self.state.pop();
    }

    fn visit_drop(&mut self, _: &Drop) {
        self.state.pop();
    }

    fn visit_select(&mut self, _: &Select) {
        self.state.pop();

        let r = self.state.pop();
        let l = self.state.pop();
        self.state.push(l.joined(&r));
    }

    fn visit_call(&mut self, c: &Call) {
        let ty_id = self.module.funcs.get(c.func).ty();
        let ty = self.module.types.get(ty_id);
        self.pop_push_n(ty.params().len(), ty.results().len());
    }

    fn visit_call_indirect(&mut self, c: &CallIndirect) {
        let ty = self.module.types.get(c.ty);
        // +1 for the function index popped before the args.
        self.pop_push_n(ty.params().len() + 1, ty.results().len());
    }

    fn visit_return_call(&mut self, c: &ReturnCall) {
        let ty_id = self.module.funcs.get(c.func).ty();
        let ty = self.module.types.get(ty_id);
        self.pop_push_n(ty.params().len(), 0);
    }

    fn visit_return_call_indirect(&mut self, c: &ReturnCallIndirect) {
        let ty = self.module.types.get(c.ty);
        self.pop_push_n(ty.params().len() + 1, 0);
    }

    fn visit_return(&mut self, _: &Return) {}

    fn visit_unreachable(&mut self, _: &Unreachable) {}

    fn visit_memory_size(&mut self, _: &MemorySize) {
        self.state.push(Provenance::new());
    }

    fn visit_memory_grow(&mut self, _: &MemoryGrow) {
        self.state.pop();
        self.state.push(Provenance::new());
    }
}
