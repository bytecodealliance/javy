use anyhow::{Result, bail};
use std::{borrow::Cow, fs, rc::Rc};
use walrus::{FunctionId, ImportKind, ValType};
use wasmparser::{Parser, Payload};
use wasmtime_wasi::WasiCtxBuilder;
use wizer::{Linker, Wizer, wasmtime::Module};

/// Extract core module if it's a component, then run wasm-opt and Wizer to
/// initialize a plugin.
pub fn initialize_plugin(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let wasm_bytes = extract_core_module_if_necessary(wasm_bytes)?;
    // Re-encode overlong indexes with wasm-opt before running Wizer.
    let wasm_bytes = optimize_module(&wasm_bytes)?;
    let wasm_bytes = preinitialize_module(&wasm_bytes)?;
    Ok(wasm_bytes)
}

/// Extracts core plugin module from a plugin component.
pub fn extract_core_module_if_necessary(wasm_bytes: &[u8]) -> Result<Cow<'_, [u8]>> {
    let is_component = Parser::is_component(wasm_bytes);
    if !is_component && !Parser::is_core_wasm(wasm_bytes) {
        bail!("Expected Wasm module or component, received unknown file type");
    }
    if !is_component {
        return Ok(wasm_bytes.into());
    }

    let wasm_bytes = extract_core_module(wasm_bytes)?;
    let wasm_bytes = strip_wasi_p2_imports(&wasm_bytes)?;
    Ok(wasm_bytes.into())
}

fn extract_core_module(component_bytes: &[u8]) -> Result<Vec<u8>> {
    let parser = Parser::new(0);

    for payload in parser.parse_all(component_bytes) {
        if let Payload::ModuleSection {
            parser,
            unchecked_range,
        } = payload?
        {
            let module_bytes = &component_bytes[unchecked_range.start..unchecked_range.end];
            let mut extract_this_module = false;
            for payload in parser.parse_all(module_bytes) {
                match payload? {
                    Payload::ExportSection(exports) => {
                        for export in exports {
                            let export = export?;
                            if export.name == "invoke" {
                                extract_this_module = true;
                                break;
                            }
                        }
                    }
                    _ => continue,
                }
            }
            if extract_this_module {
                return Ok(module_bytes.to_vec());
            }
        }
    }

    anyhow::bail!("No module with export named `invoke` found in component");
}

fn strip_wasi_p2_imports(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut module = walrus::Module::from_buffer(wasm_bytes)?;
    let wasi_p2_imports = module
        .imports
        .iter()
        .filter_map(|import| match import.kind {
            ImportKind::Function(id)
                if import.module.starts_with("wasi:") || import.name == "adapter_close_badfd" =>
            {
                Some(id)
            }
            _ => None,
        })
        .collect::<Vec<FunctionId>>();

    for import in wasi_p2_imports {
        let results = module
            .types
            .get(module.funcs.get(import).ty())
            .results()
            .to_vec();
        module.replace_imported_func(import, |(builder, _)| {
            let mut func_body = builder.func_body();
            for result in results {
                match result {
                    ValType::I32 => {
                        func_body.i32_const(0);
                    }
                    ValType::I64 => {
                        func_body.i64_const(0);
                    }
                    ValType::F32 => {
                        func_body.f32_const(0.0);
                    }
                    ValType::F64 => {
                        func_body.f64_const(0.0);
                    }
                    ValType::V128 | ValType::Ref(_) => {
                        func_body.unreachable();
                    }
                }
            }
        })?;
    }
    Ok(module.emit_wasm())
}

fn optimize_module(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let temp_dir = tempfile::tempdir()?;
    let infile = temp_dir.path().join("infile.wasm");
    fs::write(&infile, wasm_bytes)?;
    let outfile = temp_dir.path().join("outfile.wasm");
    wasm_opt::OptimizationOptions::new_opt_level_3() // Aggressively optimize for speed.
        .shrink_level(wasm_opt::ShrinkLevel::Level0) // Don't optimize for size at the expense of performance.
        .debug_info(false)
        .run(&infile, &outfile)?;
    let optimized_wasm_bytes = fs::read(outfile)?;
    Ok(optimized_wasm_bytes)
}

fn preinitialize_module(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut wizer = Wizer::new();
    let owned_wasm_bytes = wasm_bytes.to_vec();
    wizer
        .init_func("initialize-runtime")
        .keep_init_func(true)
        .make_linker(Some(Rc::new(move |engine| {
            let mut linker = Linker::new(engine);
            wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |ctx| {
                if ctx.wasi_ctx.is_none() {
                    ctx.wasi_ctx = Some(WasiCtxBuilder::new().inherit_stderr().build_p1());
                }
                ctx.wasi_ctx.as_mut().unwrap()
            })?;
            linker.define_unknown_imports_as_traps(&Module::new(engine, &owned_wasm_bytes)?)?;
            Ok(linker)
        })))?
        .run(wasm_bytes)
}
