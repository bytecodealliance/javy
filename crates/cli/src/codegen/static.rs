use std::{collections::HashMap, fs, rc::Rc, sync::OnceLock};

use anyhow::Result;
use javy_config::Config;
use walrus::{DataKind, ExportItem, FunctionBuilder, FunctionId, MemoryId, ValType};
use wasi_common::{pipe::ReadPipe, sync::WasiCtxBuilder, WasiCtx};
use wasm_opt::{OptimizationOptions, ShrinkLevel};
use wasmtime::Linker;
use wizer::Wizer;

use crate::{
    codegen::{
        exports,
        transform::{self, SourceCodeSection},
        CodeGen, CodeGenType, Exports, WitOptions,
    },
    js::JS,
};

pub(crate) struct StaticGenerator {
    /// QuickJS engine compiled to Wasm.
    engine: &'static [u8],
    /// Whether to embed the compressed JS source in the generated module.
    pub source_compression: bool,
    /// JavaScript function exports.
    function_exports: Exports,
    /// WIT options for code generation.
    pub wit_opts: WitOptions,
    /// JS runtime options for code generation.
    pub js_runtime_config: Config,
}

impl StaticGenerator {
    /// Creates a new [`StaticGenerator`].
    pub fn new(js_runtime_config: Config) -> Self {
        let engine = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
        Self {
            engine,
            source_compression: true,
            function_exports: Default::default(),
            wit_opts: Default::default(),
            js_runtime_config,
        }
    }
}

// We can't move the WasiCtx into `make_linker` since WasiCtx doesn't
// implement the `Copy` trait. So we move the WasiCtx into a mutable
// static OnceLock instead. Setting the value in the `OnceLock` and
// getting the reference back from it should be safe given we're never
// executing this code concurrently. This code will also fail if
// `generate` is invoked more than once per execution.
static mut WASI: OnceLock<WasiCtx> = OnceLock::new();

impl CodeGen for StaticGenerator {
    fn generate(&mut self, js: &JS) -> Result<Vec<u8>> {
        if self.wit_opts.defined() {
            self.function_exports = exports::process_exports(
                js,
                self.wit_opts.unwrap_path(),
                self.wit_opts.unwrap_world(),
            )?;
        }

        unsafe {
            WASI.get_or_init(|| {
                WasiCtxBuilder::new()
                    .inherit_stderr()
                    .inherit_stdout()
                    .build()
            });

            WASI.get_mut()
                .unwrap()
                .set_stdin(Box::new(ReadPipe::from(js.as_bytes())));

            WASI.get_mut().unwrap().push_env(
                "JS_RUNTIME_CONFIG",
                &self.js_runtime_config.bits().to_string(),
            )?;
        };

        let wasm = Wizer::new()
            .init_func("initialize_runtime")
            .make_linker(Some(Rc::new(|engine| {
                let mut linker = Linker::new(engine);
                wasi_common::sync::add_to_linker(&mut linker, |_: &mut Option<WasiCtx>| unsafe {
                    WASI.get_mut().unwrap()
                })?;
                Ok(linker)
            })))?
            .wasm_bulk_memory(true)
            .run(self.engine)?;

        let mut module = transform::module_config().parse(&wasm)?;

        let (realloc, free, invoke, memory) = {
            let mut exports = HashMap::new();
            for export in module.exports.iter() {
                exports.insert(export.name.as_str(), export);
            }
            (
                *exports.get("canonical_abi_realloc").unwrap(),
                *exports.get("canonical_abi_free").unwrap(),
                *exports.get("javy.invoke").unwrap(),
                *exports.get("memory").unwrap(),
            )
        };

        let realloc_export = realloc.id();
        let free_export = free.id();
        let invoke_export = invoke.id();

        if !self.function_exports.is_empty() {
            let ExportItem::Function(realloc_fn) = realloc.item else {
                unreachable!()
            };
            let ExportItem::Function(invoke_fn) = invoke.item else {
                unreachable!()
            };
            let ExportItem::Memory(memory) = memory.item else {
                unreachable!()
            };
            export_exported_js_functions(
                &mut module,
                realloc_fn,
                invoke_fn,
                memory,
                &self.function_exports,
            );
        }

        // We no longer need these exports so remove them.
        module.exports.delete(realloc_export);
        module.exports.delete(free_export);
        module.exports.delete(invoke_export);

        let wasm = module.emit_wasm();

        let wasm = optimize_wasm(&wasm)?;

        let mut module = transform::module_config().parse(&wasm)?;
        if !self.source_compression {
            module.customs.add(SourceCodeSection::uncompressed(js)?);
        } else {
            module.customs.add(SourceCodeSection::compressed(js)?);
        }
        transform::add_producers_section(&mut module.producers);
        Ok(module.emit_wasm())
    }

    fn classify() -> CodeGenType {
        CodeGenType::Static
    }
}

fn export_exported_js_functions(
    module: &mut walrus::Module,
    realloc_fn: FunctionId,
    invoke_fn: FunctionId,
    memory: MemoryId,
    js_exports: &Exports,
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

fn optimize_wasm(wasm: &[u8]) -> Result<Vec<u8>> {
    let tempdir = tempfile::tempdir()?;
    let tempfile_path = tempdir.path().join("temp.wasm");

    fs::write(&tempfile_path, wasm)?;

    OptimizationOptions::new_opt_level_3() // Aggressively optimize for speed.
        .shrink_level(ShrinkLevel::Level0) // Don't optimize for size at the expense of performance.
        .debug_info(false)
        .run(&tempfile_path, &tempfile_path)?;

    Ok(fs::read(&tempfile_path)?)
}

#[cfg(test)]
mod test {
    use super::StaticGenerator;
    use super::WitOptions;
    use anyhow::Result;
    use javy_config::Config;

    #[test]
    fn default_values() -> Result<()> {
        let gen = StaticGenerator::new(Config::default());
        assert!(gen.source_compression);
        assert_eq!(gen.wit_opts, WitOptions::default());

        Ok(())
    }
}
