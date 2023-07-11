use std::collections::HashMap;

use anyhow::{anyhow, Result};
use binaryen::{CodegenConfig, Module};
use walrus::{DataKind, ExportItem, FunctionBuilder, FunctionId, MemoryId, ValType};
use wizer::Wizer;

use crate::{exports::Export, js::JS};

use super::transform::{self, SourceCodeSection};

/// Generates Wasm for a static Javy module within a subprocess.
///
/// We assume stdin contains the JS source code.
pub fn generate() -> Result<Vec<u8>> {
    let wasm = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
    let wasm = Wizer::new()
        .allow_wasi(true)?
        .inherit_stdio(true)
        .wasm_bulk_memory(true)
        .run(wasm)
        .map_err(|_| anyhow!("JS compilation failed"))?;
    Ok(wasm)
}

/// Takes Wasm created by `Generator` and makes additional changes.
///
/// This is intended to be run in the parent process after generating the Wasm.
pub fn refine(wasm: Vec<u8>, js: &JS, exports: Vec<Export>) -> Result<Vec<u8>> {
    let mut module = transform::module_config().parse(&wasm)?;

    let (realloc, invoke, memory) = {
        let mut exports = HashMap::new();
        for export in module.exports.iter() {
            exports.insert(export.name.as_str(), export);
        }
        (
            *exports.get("canonical_abi_realloc").unwrap(),
            *exports.get("javy.invoke").unwrap(),
            *exports.get("memory").unwrap(),
        )
    };

    let realloc_export = realloc.id();
    let invoke_export = invoke.id();

    if !exports.is_empty() {
        let ExportItem::Function(realloc_fn) = realloc.item else { unreachable!() };
        let ExportItem::Function(invoke_fn) = invoke.item else { unreachable!() };
        let ExportItem::Memory(memory) = memory.item else { unreachable!() };
        export_exported_js_functions(&mut module, realloc_fn, invoke_fn, memory, exports);
    }

    // We no longer need these exports so remove them.
    module.exports.delete(realloc_export);
    module.exports.delete(invoke_export);

    let wasm = module.emit_wasm();

    let codegen_cfg = CodegenConfig {
        optimization_level: 3, // Aggressively optimize for speed.
        shrink_level: 0,       // Don't optimize for size at the expense of performance.
        debug_info: false,
    };

    let mut module = Module::read(&wasm)
        .map_err(|_| anyhow!("Unable to read wasm binary for wasm-opt optimizations"))?;
    module.optimize(&codegen_cfg);
    module
        .run_optimization_passes(vec!["strip"], &codegen_cfg)
        .map_err(|_| anyhow!("Running wasm-opt optimization passes failed"))?;
    let wasm = module.write();

    let mut module = transform::module_config().parse(&wasm)?;
    module.customs.add(SourceCodeSection::new(js)?);
    transform::add_producers_section(&mut module.producers);
    Ok(module.emit_wasm())
}

fn export_exported_js_functions(
    module: &mut walrus::Module,
    realloc_fn: FunctionId,
    invoke_fn: FunctionId,
    memory: MemoryId,
    js_exports: Vec<Export>,
) {
    let ptr_local = module.locals.add(ValType::I32);
    for export in js_exports {
        // For each JS function export, add an export that copies the name of the function into memory and invokes it.
        let js_export_bytes = export.js.as_bytes();
        let js_export_len: i32 = js_export_bytes.len().try_into().unwrap();
        let fn_name_data = module.data.add(DataKind::Passive, js_export_bytes.to_vec());

        let mut export_fn = FunctionBuilder::new(&mut module.types, &[], &[]);
        export_fn
            .func_body()
            .i32_const(0) // orig ptr
            .i32_const(0) // orig len
            .i32_const(1) // alignment
            .i32_const(js_export_len) // new size
            .call(realloc_fn)
            .local_tee(ptr_local)
            .i32_const(0) // offset into data segment
            .i32_const(js_export_len) // size to copy
            .memory_init(memory, fn_name_data) // copy fn name into allocated memory
            .data_drop(fn_name_data)
            .local_get(ptr_local)
            .i32_const(js_export_len)
            .call(invoke_fn);
        let export_fn = export_fn.finish(vec![], &mut module.funcs);
        module.exports.add(&export.wit, export_fn);
    }
}
