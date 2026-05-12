//! Instrumentation state to derive probe insertion.

use anyhow::{Result, bail};
use std::collections::BTreeSet;
use walrus::ir::{BrTable, Visitor, dfs_in_order};
use walrus::{FunctionId, FunctionKind, LocalFunction, ModuleConfig};

use crate::ai;

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
        let dispatch_loads = ai::analyze(&module, local, threshold);

        Ok(Self {
            dispatch_func_idx,
            dispatch_loads,
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
    use walrus::ir::Instr;

    fn to_bytes(wat: &str) -> Result<Vec<u8>> {
        Ok(wat::parse_str(wat)?)
    }

    /// Map every instruction's byte offset to the corresponding
    /// instruction in the dispatch function.
    fn pc2instr(state: &State) -> Result<HashMap<u32, Instr>> {
        let func = state
            .module()
            .funcs
            .iter()
            .nth(state.dispatch_func_idx as usize)
            .ok_or_else(|| anyhow!("no function at index {}", state.dispatch_func_idx))?;
        let local = match &func.kind {
            FunctionKind::Local(l) => l,
            _ => bail!(
                "function at index {} is not a local function",
                state.dispatch_func_idx
            ),
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

    fn assert_all_byte_loads(state: &State) -> Result<()> {
        let map = pc2instr(state)?;
        for &pc in &state.dispatch_loads {
            let instr = map
                .get(&pc)
                .ok_or_else(|| anyhow!("pc {pc} not found in function"))?;
            match instr {
                Instr::Load(l) => {
                    if !ai::is_byte_load(l) {
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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

        assert!(state.is_dispatch_func(state.dispatch_func_idx));
        assert_eq!(state.dispatch_loads.len(), 1, "expected one dispatch load");
        assert_all_byte_loads(&state)?;
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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

        assert_eq!(
            state.dispatch_loads.len(),
            1,
            "and must not drop provenance"
        );
        assert_all_byte_loads(&state)?;
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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

        assert_eq!(state.dispatch_loads.len(), 1);
        assert_all_byte_loads(&state)?;
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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

        assert_eq!(state.dispatch_loads.len(), 2, "both loads must be recorded");
        assert_all_byte_loads(&state)?;
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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

        assert_eq!(state.dispatch_loads.len(), 2);
        assert_all_byte_loads(&state)?;
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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

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
        let state = State::from_bytes_with_threshold(&to_bytes(&wat)?, 3)?;

        let pc = *state.dispatch_loads.iter().next().unwrap();
        assert!(state.is_dispatch_load(state.dispatch_func_idx, pc));
        assert!(!state.is_dispatch_load(state.dispatch_func_idx + 1, pc));
        Ok(())
    }
}
