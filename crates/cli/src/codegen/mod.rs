//! WebAssembly Code Generation for JavaScript
//!
//! This module provides all the functionality to emit Wasm modules for
//! a particular JavaScript program.
//!
//! Javy supports two main code generation paths:
//!
//! 1. Static code generation
//! 2. Dynamic code generation
//!
//!
//! ## Static code generation
//!
//! A single unit of code is generated, which is a Wasm module consisting of the
//! bytecode representation of a given JavaScript program and the code for
//! a particular version of the QuickJS engine compiled to Wasm.
//!
//! The generated Wasm module is self contained and the bytecode version matches
//! the exact requirements of the embedded QuickJs engine.
//!
//! ## Dynamic code generation
//!
//! A single unit of code is generated, which is a Wasm module consisting of the
//! bytecode representation of a given JavaScript program. The JavaScript
//! bytecode is stored as part of the data section of the module which also
//! contains instructions to execute that bytecode through dynamic linking
//! at runtime.
//!
//! Dynamic code generation requires a provider module to be used and linked
//! against at runtime in order to execute the JavaScript bytecode. This
//! operation involves carefully ensuring that a given provider version matches
//! the provider version of the imports requested by the generated Wasm module
//! as well as ensuring that any features available in the provider match the
//! features requsted by the JavaScript bytecode.

mod builder;
use std::{env, rc::Rc, sync::OnceLock};

pub(crate) use builder::*;

mod transform;

mod exports;
pub(crate) use exports::*;
use javy_config::Config;
use transform::SourceCodeSection;
use wasi_common::{sync::WasiCtxBuilder, WasiCtx};
use wizer::{Linker, Wizer};

use crate::JS;

pub(crate) trait CodeGen {
    /// Generate Wasm from a given JS source.
    fn generate(&mut self, source: &JS) -> anyhow::Result<Vec<u8>>;
}

pub(crate) use crate::codegen::WitOptions;
use anyhow::Result;
use walrus::{
    DataId, DataKind, ExportItem, FunctionBuilder, FunctionId, LocalId, MemoryId, Module, ValType,
};

/// Helper struct to keep track of bytecode metadata.
// This is an internal detail of this module.
pub(crate) struct BytecodeMetadata {
    ptr: LocalId,
    len: i32,
    data_section: DataId,
}

impl BytecodeMetadata {
    fn new(ptr: LocalId, len: i32, data_section: DataId) -> Self {
        Self {
            ptr,
            len,
            data_section,
        }
    }
}

struct FunctionsAndMemory {
    canonical_abi_realloc: FunctionId,
    invoke: FunctionId,
    memory: MemoryId,
}

enum LinkingStrategy {
    Static { js_runtime_config: Config },
    Dynamic { import_namespace: String },
}

static mut WASI: OnceLock<WasiCtx> = OnceLock::new();

impl LinkingStrategy {
    fn module(&self) -> Result<walrus::Module> {
        let config = transform::module_config();
        let module = match self {
            LinkingStrategy::Static { js_runtime_config } => {
                unsafe {
                    WASI.get_or_init(|| {
                        WasiCtxBuilder::new()
                            .inherit_stderr()
                            .inherit_stdout()
                            .env("JS_RUNTIME_CONFIG", &js_runtime_config.bits().to_string())
                            .unwrap()
                            .build()
                    });
                };
                let wasm = Wizer::new()
                    .make_linker(Some(Rc::new(|engine| {
                        let mut linker = Linker::new(engine);
                        wasi_common::sync::add_to_linker(&mut linker, |_| unsafe {
                            WASI.get_mut().unwrap()
                        })?;
                        Ok(linker)
                    })))?
                    .wasm_bulk_memory(true)
                    .init_func("initialize_runtime")
                    .run(include_bytes!(concat!(env!("OUT_DIR"), "/provider.wasm")))?;
                let module = config.parse(&wasm)?;
                module
            }
            LinkingStrategy::Dynamic { .. } => walrus::Module::with_config(config),
        };
        Ok(module)
    }

    fn functions_and_memory(&self, module: &mut walrus::Module) -> Result<FunctionsAndMemory> {
        match self {
            LinkingStrategy::Static { .. } => {
                let canonical_abi_realloc_fn_id =
                    module.exports.get_func("canonical_abi_realloc")?;
                let invoke_id = module.exports.get_func("invoke")?;
                let ExportItem::Memory(memory) = module
                    .exports
                    .iter()
                    .find(|e| e.name == "memory")
                    .ok_or_else(|| anyhow::anyhow!("Missing memory export"))?
                    .item
                else {
                    anyhow::bail!("Export with name memory must be of type memory")
                };
                Ok(FunctionsAndMemory {
                    canonical_abi_realloc: canonical_abi_realloc_fn_id,
                    invoke: invoke_id,
                    memory,
                })
            }
            LinkingStrategy::Dynamic { import_namespace } => {
                let canonical_abi_realloc_type = module.types.add(
                    &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                    &[ValType::I32],
                );
                let (canonical_abi_realloc_fn_id, _) = module.add_import_func(
                    &import_namespace,
                    "canonical_abi_realloc",
                    canonical_abi_realloc_type,
                );

                let invoke_type = module.types.add(
                    &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                    &[],
                );
                let (invoke_id, _) =
                    module.add_import_func(&import_namespace, "invoke", invoke_type);

                let (memory_id, _) = module.add_import_memory(
                    &import_namespace,
                    "memory",
                    false,
                    false,
                    0,
                    None,
                    None,
                );
                Ok(FunctionsAndMemory {
                    canonical_abi_realloc: canonical_abi_realloc_fn_id,
                    invoke: invoke_id,
                    memory: memory_id,
                })
            }
        }
    }

    fn cleanup(&self, module: &mut walrus::Module) -> Result<()> {
        match self {
            LinkingStrategy::Static { .. } => {
                module.exports.remove("canonical_abi_realloc")?;
                module.exports.remove("invoke")?;
                module.exports.remove("compile_src")?;
                Ok(())
            }
            LinkingStrategy::Dynamic { .. } => Ok(()),
        }
    }
}

/// Code Generation.
pub(crate) struct Generator {
    linking_strategy: LinkingStrategy,
    /// JavaScript function exports.
    function_exports: Exports,
    /// Whether to embed the compressed JS source in the generated module.
    pub source_compression: bool,
    /// WIT options for code generation.
    wit_opts: WitOptions,
}

impl Generator {
    /// Creates a new [`Generator`].
    fn new(linking_strategy: LinkingStrategy, wit_opts: WitOptions) -> Self {
        Self {
            linking_strategy,
            source_compression: true,
            function_exports: Default::default(),
            wit_opts,
        }
    }

    /// Generate the main function.
    fn generate_main(
        &self,
        module: &mut Module,
        js: &JS,
        functions_and_memory: &FunctionsAndMemory,
    ) -> Result<BytecodeMetadata> {
        let bytecode = js.compile()?;
        let bytecode_len: i32 = bytecode.len().try_into()?;
        let bytecode_data = module.data.add(DataKind::Passive, bytecode);

        let mut main = FunctionBuilder::new(&mut module.types, &[], &[]);
        let bytecode_ptr_local = module.locals.add(ValType::I32);
        main.func_body()
            // Allocate memory in javy_quickjs_provider for bytecode array.
            .i32_const(0) // orig ptr
            .i32_const(0) // orig size
            .i32_const(1) // alignment
            .i32_const(bytecode_len) // new size
            .call(functions_and_memory.canonical_abi_realloc)
            // Copy bytecode array into allocated memory.
            .local_tee(bytecode_ptr_local) // save returned address to local and set as dest addr for mem.init
            .i32_const(0) // offset into data segment for mem.init
            .i32_const(bytecode_len) // size to copy from data segment
            // top-2: dest addr, top-1: offset into source, top-0: size of memory region in bytes.
            .memory_init(functions_and_memory.memory, bytecode_data)
            // Invoke top-level scope.
            .local_get(bytecode_ptr_local) // ptr to bytecode
            .i32_const(bytecode_len)
            .i32_const(0) // ptr to function name to call
            .i32_const(0) // length of function name to call
            .call(functions_and_memory.invoke);
        let main = main.finish(vec![], &mut module.funcs);

        module.exports.add("_start", main);
        Ok(BytecodeMetadata::new(
            bytecode_ptr_local,
            bytecode_len,
            bytecode_data,
        ))
    }

    /// Generate function exports.
    fn generate_exports(
        &self,
        module: &mut Module,
        functions_and_memory: &FunctionsAndMemory,
        bc_metadata: &BytecodeMetadata,
    ) -> Result<()> {
        if !self.function_exports.is_empty() {
            // let invoke_type = module.types.add(
            //     &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            //     &[],
            // );

            let fn_name_ptr_local = module.locals.add(ValType::I32);
            for export in &self.function_exports {
                // For each JS function export, add an export that copies the name of the function into memory and invokes it.
                let js_export_bytes = export.js.as_bytes();
                let js_export_len: i32 = js_export_bytes.len().try_into().unwrap();
                let fn_name_data = module.data.add(DataKind::Passive, js_export_bytes.to_vec());

                let mut export_fn = FunctionBuilder::new(&mut module.types, &[], &[]);
                export_fn
                    .func_body()
                    // Copy bytecode.
                    .i32_const(0) // orig ptr
                    .i32_const(0) // orig len
                    .i32_const(1) // alignment
                    .i32_const(bc_metadata.len) // size to copy
                    .call(functions_and_memory.canonical_abi_realloc)
                    .local_tee(bc_metadata.ptr)
                    .i32_const(0) // offset into data segment
                    .i32_const(bc_metadata.len) // size to copy
                    .memory_init(functions_and_memory.memory, bc_metadata.data_section) // copy bytecode into allocated memory
                    .data_drop(bc_metadata.data_section)
                    // Copy function name.
                    .i32_const(0) // orig ptr
                    .i32_const(0) // orig len
                    .i32_const(1) // alignment
                    .i32_const(js_export_len) // new size
                    .call(functions_and_memory.canonical_abi_realloc)
                    .local_tee(fn_name_ptr_local)
                    .i32_const(0) // offset into data segment
                    .i32_const(js_export_len) // size to copy
                    .memory_init(functions_and_memory.memory, fn_name_data) // copy fn name into allocated memory
                    .data_drop(fn_name_data)
                    // Call invoke.
                    .local_get(bc_metadata.ptr)
                    .i32_const(bc_metadata.len)
                    .local_get(fn_name_ptr_local)
                    .i32_const(js_export_len)
                    .call(functions_and_memory.invoke);
                let export_fn = export_fn.finish(vec![], &mut module.funcs);
                module.exports.add(&export.wit, export_fn);
            }
        }
        Ok(())
    }
}

impl CodeGen for Generator {
    // Run the calling code with the `dump_wat` feature enabled to print the WAT to stdout
    //
    // For the example generated WAT, the `bytecode_len` is 137
    // (module
    //    (type (;0;) (func))
    //    (type (;1;) (func (param i32 i32)))
    //    (type (;2;) (func (param i32 i32 i32 i32)))
    //    (type (;3;) (func (param i32 i32 i32 i32) (result i32)))
    //    (import "javy_quickjs_provider_v2" "canonical_abi_realloc" (func (;0;) (type 3)))
    //    (import "javy_quickjs_provider_v2" "eval_bytecode" (func (;1;) (type 1)))
    //    (import "javy_quickjs_provider_v2" "memory" (memory (;0;) 0))
    //    (import "javy_quickjs_provider_v2" "invoke" (func (;2;) (type 2)))
    //    (func (;3;) (type 0)
    //      (local i32 i32)
    //      i32.const 0
    //      i32.const 0
    //      i32.const 1
    //      i32.const 137
    //      call 0
    //      local.tee 0
    //      i32.const 0
    //      i32.const 137
    //      memory.init 0
    //      data.drop 0
    //      i32.const 0
    //      i32.const 0
    //      i32.const 1
    //      i32.const 3
    //      call 0
    //      local.tee 1
    //      i32.const 0
    //      i32.const 3
    //      memory.init 1
    //      data.drop 1
    //      local.get 0
    //      i32.const 137
    //      local.get 1
    //      i32.const 3
    //      call 2
    //    )
    //    (func (;4;) (type 0)
    //      (local i32)
    //      i32.const 0
    //      i32.const 0
    //      i32.const 1
    //      i32.const 137
    //      call 0
    //      local.tee 0
    //      i32.const 0
    //      i32.const 137
    //      memory.init 0
    //      local.get 0
    //      i32.const 137
    //      call 1
    //    )
    //    (export "_start" (func 4))
    //    (export "foo" (func 3))
    //    (data (;0;) "\02\05\18function.mjs\06foo\0econsole\06log\06bar\0f\bc\03\00\01\00\00\be\03\00\00\0e\00\06\01\a0\01\00\00\00\03\01\01\1a\00\be\03\00\01\08\ea\05\c0\00\e1)8\e0\00\00\00B\e1\00\00\00\04\e2\00\00\00$\01\00)\bc\03\01\04\01\00\07\0a\0eC\06\01\be\03\00\00\00\03\00\00\13\008\e0\00\00\00B\e1\00\00\00\04\df\00\00\00$\01\00)\bc\03\01\02\03]")
    //    (data (;1;) "foo")
    //  )
    fn generate(&mut self, js: &JS) -> Result<Vec<u8>> {
        if self.wit_opts.defined() {
            self.function_exports = exports::process_exports(
                js,
                self.wit_opts.unwrap_path(),
                self.wit_opts.unwrap_world(),
            )?;
        }

        let mut module = self.linking_strategy.module()?;
        let functions_and_memory = self.linking_strategy.functions_and_memory(&mut module)?;
        let bc_metadata = self.generate_main(&mut module, js, &functions_and_memory)?;
        self.generate_exports(&mut module, &functions_and_memory, &bc_metadata)?;
        self.linking_strategy.cleanup(&mut module)?;

        transform::add_producers_section(&mut module.producers);
        if !self.source_compression {
            module.customs.add(SourceCodeSection::uncompressed(js)?);
        } else {
            module.customs.add(SourceCodeSection::compressed(js)?);
        }

        let wasm = module.emit_wasm();
        print_wat(&wasm)?;
        Ok(wasm)
    }
}

#[cfg(feature = "dump_wat")]
fn print_wat(wasm_binary: &[u8]) -> Result<()> {
    println!(
        "Generated WAT: \n{}",
        wasmprinter::print_bytes(&wasm_binary)?
    );
    Ok(())
}

#[cfg(not(feature = "dump_wat"))]
fn print_wat(_wasm_binary: &[u8]) -> Result<()> {
    Ok(())
}
