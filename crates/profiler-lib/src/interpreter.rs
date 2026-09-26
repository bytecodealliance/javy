//! Abstract interpretation for dispatch `br_table` provenance.
//!
//! Given a WebAssembly module and a function, determine which byte loads
//! contribute to the index of a wide `br_table`.
//!
//! This walks [`wirm`], the same IR whamm itself uses, so that every offset we
//! report is already in whamm's `pc` coordinate. THIS IS FRAGILE, since using
//! any other library will result in `pc` coordinates not matching.

use anyhow::{anyhow, Result};
use std::collections::{BTreeSet, HashMap};
use wirm::ir::id::{FunctionID, TypeID};
use wirm::ir::module::module_functions::{FuncKind, LocalFunction};
use wirm::ir::module::module_types::Types;
use wirm::wasmparser::Operator;
use wirm::Module;

/// whamm reports `pc` as wirm's function body-relative instruction offset plus
/// one, to match Wizard's convention of pointing just past the opcode. See
/// `whamm::emitter::rewriting::visiting_emitter::VisitingEmitter::lookup_pc_offset_for`.
/// This is not documented in whamm's provider definitions, so
/// `tests::whamm_pc_is_body_relative_plus_one` pins it.
const WHAMM_PC_OFFSET: u32 = 1;

/// The `pc` whamm will bind for the instruction at `instr_idx`.
pub(crate) fn whamm_pc(local: &LocalFunction, instr_idx: usize) -> Result<u32> {
    local
        .lookup_pc_offset_for(instr_idx)
        .map(|offset| offset as u32 + WHAMM_PC_OFFSET)
        .ok_or_else(|| {
            anyhow!(
                "no recorded offset for instruction {instr_idx}; the module must \
                 be parsed with offsets enabled"
            )
        })
}

/// The set of byte-load instructions that contribute to a value, identified by
/// the `pc` whamm binds for them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Provenance(BTreeSet<u32>);

impl Provenance {
    fn new() -> Self {
        Self(BTreeSet::new())
    }

    fn with(pc: u32) -> Self {
        let mut s = BTreeSet::new();
        s.insert(pc);
        Self(s)
    }

    fn join_in_place(&mut self, other: &Provenance) {
        self.0.extend(other.0.iter().copied());
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
    locals: HashMap<u32, Provenance>,
}

impl AbstractState {
    fn join(&mut self, other: &AbstractState) {
        // The spec guarantees matching stack heights at merge points; take the
        // shorter to stay safe if our model has drifted.
        let n = self.stack.len().min(other.stack.len());
        for i in 0..n {
            let addend = other.stack[i].clone();
            self.stack[i].join_in_place(&addend);
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

    fn pop_push(&mut self, pops: usize, pushes: usize) {
        for _ in 0..pops {
            self.pop();
        }
        for _ in 0..pushes {
            self.push(Provenance::new());
        }
    }
}

/// A structured control frame, reconstructed from the flat operator stream.
#[derive(Debug)]
enum Frame {
    Block {
        target: AbstractState,
    },
    Loop {
        header: AbstractState,
    },
    If {
        entry: AbstractState,
        target: AbstractState,
    },
    Else {
        if_exit: AbstractState,
        target: AbstractState,
    },
}

/// Byte loads whose values reach the index of a `br_table` with at least
/// `threshold` distinct targets, as whamm `pc`s.
pub(crate) fn analyze(
    module: &Module,
    local: &LocalFunction,
    threshold: u32,
) -> Result<BTreeSet<u32>> {
    let mut interp = AbstractInterp {
        module,
        local,
        state: AbstractState::default(),
        // The function body behaves as an enclosing block.
        frames: vec![Frame::Block {
            target: AbstractState::default(),
        }],
        dispatch_loads: BTreeSet::new(),
        threshold: threshold as usize,
    };
    interp.run()?;
    Ok(interp.dispatch_loads)
}

/// Byte offsets, as whamm `pc`s, of the instructions in `local` that the
/// profiler counts as executed work. Structural opcodes are excluded: they
/// delimit control flow rather than performing any.
pub(crate) fn countable_opcodes(local: &LocalFunction) -> Result<BTreeSet<u32>> {
    let mut out = BTreeSet::new();
    for (idx, op) in local.body.instructions.get_ops().iter().enumerate() {
        if is_structural(op) {
            continue;
        }
        out.insert(whamm_pc(local, idx)?);
    }
    Ok(out)
}

/// Opcodes that delimit control flow or discard a value rather than doing work.
fn is_structural(op: &Operator) -> bool {
    matches!(
        op,
        Operator::Block { .. }
            | Operator::Loop { .. }
            | Operator::End
            | Operator::Else
            | Operator::Nop
            | Operator::Drop
            | Operator::Return
            | Operator::Unreachable
    )
}

pub(crate) fn is_byte_load(op: &Operator) -> bool {
    matches!(
        op,
        Operator::I32Load8U { .. }
            | Operator::I32Load8S { .. }
            | Operator::I64Load8U { .. }
            | Operator::I64Load8S { .. }
    )
}

fn is_load(op: &Operator) -> bool {
    matches!(
        op,
        Operator::I32Load { .. }
            | Operator::I64Load { .. }
            | Operator::F32Load { .. }
            | Operator::F64Load { .. }
            | Operator::I32Load8S { .. }
            | Operator::I32Load8U { .. }
            | Operator::I32Load16S { .. }
            | Operator::I32Load16U { .. }
            | Operator::I64Load8S { .. }
            | Operator::I64Load8U { .. }
            | Operator::I64Load16S { .. }
            | Operator::I64Load16U { .. }
            | Operator::I64Load32S { .. }
            | Operator::I64Load32U { .. }
    )
}

fn is_store(op: &Operator) -> bool {
    matches!(
        op,
        Operator::I32Store { .. }
            | Operator::I64Store { .. }
            | Operator::F32Store { .. }
            | Operator::F64Store { .. }
            | Operator::I32Store8 { .. }
            | Operator::I32Store16 { .. }
            | Operator::I64Store8 { .. }
            | Operator::I64Store16 { .. }
            | Operator::I64Store32 { .. }
    )
}

fn is_const(op: &Operator) -> bool {
    matches!(
        op,
        Operator::I32Const { .. }
            | Operator::I64Const { .. }
            | Operator::F32Const { .. }
            | Operator::F64Const { .. }
    )
}

fn is_binop(op: &Operator) -> bool {
    matches!(
        op,
        Operator::I32Add
            | Operator::I32Sub
            | Operator::I32Mul
            | Operator::I32DivS
            | Operator::I32DivU
            | Operator::I32RemS
            | Operator::I32RemU
            | Operator::I32And
            | Operator::I32Or
            | Operator::I32Xor
            | Operator::I32Shl
            | Operator::I32ShrS
            | Operator::I32ShrU
            | Operator::I32Rotl
            | Operator::I32Rotr
            | Operator::I32Eq
            | Operator::I32Ne
            | Operator::I32LtS
            | Operator::I32LtU
            | Operator::I32GtS
            | Operator::I32GtU
            | Operator::I32LeS
            | Operator::I32LeU
            | Operator::I32GeS
            | Operator::I32GeU
            | Operator::I64Add
            | Operator::I64Sub
            | Operator::I64Mul
            | Operator::I64DivS
            | Operator::I64DivU
            | Operator::I64RemS
            | Operator::I64RemU
            | Operator::I64And
            | Operator::I64Or
            | Operator::I64Xor
            | Operator::I64Shl
            | Operator::I64ShrS
            | Operator::I64ShrU
            | Operator::I64Rotl
            | Operator::I64Rotr
            | Operator::I64Eq
            | Operator::I64Ne
            | Operator::I64LtS
            | Operator::I64LtU
            | Operator::I64GtS
            | Operator::I64GtU
            | Operator::I64LeS
            | Operator::I64LeU
            | Operator::I64GeS
            | Operator::I64GeU
    )
}

fn is_unop(op: &Operator) -> bool {
    matches!(
        op,
        Operator::I32Eqz
            | Operator::I32Clz
            | Operator::I32Ctz
            | Operator::I32Popcnt
            | Operator::I32Extend8S
            | Operator::I32Extend16S
            | Operator::I32WrapI64
            | Operator::I64Eqz
            | Operator::I64Clz
            | Operator::I64Ctz
            | Operator::I64Popcnt
            | Operator::I64Extend8S
            | Operator::I64Extend16S
            | Operator::I64Extend32S
            | Operator::I64ExtendI32S
            | Operator::I64ExtendI32U
    )
}

struct AbstractInterp<'a, 'b> {
    module: &'a Module<'b>,
    local: &'a LocalFunction<'b>,
    state: AbstractState,
    frames: Vec<Frame>,
    dispatch_loads: BTreeSet<u32>,
    threshold: usize,
}

impl AbstractInterp<'_, '_> {
    fn run(&mut self) -> Result<()> {
        let ops = self.local.body.instructions.get_ops();
        for (idx, op) in ops.iter().enumerate() {
            self.step(idx, op)?;
        }
        Ok(())
    }

    fn step(&mut self, idx: usize, op: &Operator) -> Result<()> {
        match op {
            Operator::Block { .. } => self.frames.push(Frame::Block {
                target: AbstractState::default(),
            }),
            Operator::Loop { .. } => self.frames.push(Frame::Loop {
                header: self.state.clone(),
            }),
            Operator::If { .. } => {
                self.state.pop();
                self.frames.push(Frame::If {
                    entry: self.state.clone(),
                    target: AbstractState::default(),
                });
            }
            Operator::Else => match self.frames.pop() {
                Some(Frame::If { entry, target }) => {
                    let if_exit = std::mem::replace(&mut self.state, entry);
                    self.frames.push(Frame::Else { if_exit, target });
                }
                other => {
                    return Err(anyhow!("`else` outside of an `if`: {other:?}"));
                }
            },
            Operator::End => match self.frames.pop() {
                Some(Frame::Block { target }) => self.state.join(&target),
                Some(Frame::Loop { .. }) => {}
                Some(Frame::If { entry, target }) => {
                    self.state.join(&entry);
                    self.state.join(&target);
                }
                Some(Frame::Else { if_exit, target }) => {
                    self.state.join(&if_exit);
                    self.state.join(&target);
                }
                None => {}
            },
            Operator::Br { relative_depth } => self.branch(*relative_depth),
            Operator::BrIf { relative_depth } => {
                self.state.pop();
                self.branch(*relative_depth);
            }
            Operator::BrTable { targets } => {
                let index = self.state.pop();
                let depths: BTreeSet<u32> = targets.targets().collect::<Result<_, _>>()?;
                if depths.len() >= self.threshold {
                    // This is a dispatch. Whatever byte loads produced the
                    // index are the opcode fetches we want to probe.
                    self.dispatch_loads.extend(index.0.iter().copied());
                }
                for depth in depths.iter().chain(std::iter::once(&targets.default())) {
                    self.branch(*depth);
                }
            }
            Operator::LocalGet { local_index } => {
                let p = self
                    .state
                    .locals
                    .get(local_index)
                    .cloned()
                    .unwrap_or_default();
                self.state.push(p);
            }
            Operator::LocalSet { local_index } => {
                let v = self.state.pop();
                self.state.locals.insert(*local_index, v);
            }
            Operator::LocalTee { local_index } => {
                let v = self.state.stack.last().cloned().unwrap_or_default();
                self.state.locals.insert(*local_index, v);
            }
            Operator::GlobalGet { .. } => self.state.push(Provenance::new()),
            Operator::GlobalSet { .. } => {
                self.state.pop();
            }
            Operator::Drop => {
                self.state.pop();
            }
            Operator::Select | Operator::TypedSelect { .. } => {
                self.state.pop();
                let r = self.state.pop();
                let l = self.state.pop();
                self.state.push(l.joined(&r));
            }
            Operator::Call { function_index } => {
                let (params, results) = self.func_arity(FunctionID(*function_index))?;
                self.state.pop_push(params, results);
            }
            Operator::ReturnCall { function_index } => {
                let (params, _) = self.func_arity(FunctionID(*function_index))?;
                self.state.pop_push(params, 0);
            }
            Operator::CallIndirect { type_index, .. } => {
                let (params, results) = self.type_arity(TypeID(*type_index))?;
                // Plus the table index popped before the arguments.
                self.state.pop_push(params + 1, results);
            }
            Operator::ReturnCallIndirect { type_index, .. } => {
                let (params, _) = self.type_arity(TypeID(*type_index))?;
                self.state.pop_push(params + 1, 0);
            }
            Operator::MemorySize { .. } => self.state.push(Provenance::new()),
            Operator::MemoryGrow { .. } => {
                self.state.pop();
                self.state.push(Provenance::new());
            }
            Operator::Return | Operator::Unreachable | Operator::Nop => {}
            op if is_load(op) => {
                self.state.pop();
                if is_byte_load(op) {
                    self.state
                        .push(Provenance::with(whamm_pc(self.local, idx)?));
                } else {
                    self.state.push(Provenance::new());
                }
            }
            op if is_store(op) => self.state.pop_push(2, 0),
            op if is_const(op) => self.state.push(Provenance::new()),
            op if is_binop(op) => {
                let rhs = self.state.pop();
                let lhs = self.state.pop();
                self.state.push(lhs.joined(&rhs));
            }
            op if is_unop(op) => {
                let v = self.state.pop();
                self.state.push(v);
            }
            // Anything else leaves the abstract stack alone. Provenance only
            // has to be exact along the path from an opcode fetch to a dispatch
            // index, which is integer arithmetic on loaded bytes; float and
            // vector operations never appear there. `State` cross-checks that
            // every recorded pc really is a byte load, so a modeling gap
            // surfaces as an error rather than a wrong profile.
            _ => {}
        }
        Ok(())
    }

    /// Merge the current state into the frame `relative_depth` levels up.
    fn branch(&mut self, relative_depth: u32) {
        let snapshot = self.state.clone();
        let Some(index) = self.frames.len().checked_sub(1 + relative_depth as usize) else {
            // A branch out of the function body.
            return;
        };
        match &mut self.frames[index] {
            Frame::Loop { header } => header.join(&snapshot),
            Frame::Block { target } | Frame::If { target, .. } | Frame::Else { target, .. } => {
                target.join(&snapshot)
            }
        }
    }

    fn func_arity(&self, id: FunctionID) -> Result<(usize, usize)> {
        let ty_id = match self.module.functions.get_kind(id) {
            FuncKind::Local(l) => l.ty_id,
            FuncKind::Import(i) => i.ty_id,
        };
        self.type_arity(ty_id)
    }

    fn type_arity(&self, id: TypeID) -> Result<(usize, usize)> {
        match self.module.types.get(id) {
            Some(Types::FuncType {
                params, results, ..
            }) => Ok((params.len(), results.len())),
            other => Err(anyhow!("type {id:?} is not a function type: {other:?}")),
        }
    }
}
