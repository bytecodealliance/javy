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
//! Dynamic code generation requires a plugin module to be used and linked
//! against at runtime in order to execute the JavaScript bytecode. This
//! operation involves carefully ensuring that a given plugin version matches
//! the plugin version of the imports requested by the generated Wasm module
//! as well as ensuring that any features available in the plugin match the
//! features requsted by the JavaScript bytecode.

mod builder;
use std::{fs, rc::Rc, sync::OnceLock};

pub(crate) use builder::*;

mod transform;

mod exports;
pub(crate) use exports::*;
use transform::SourceCodeSection;
use walrus::{
    DataId, DataKind, ExportItem, FunctionBuilder, FunctionId, LocalId, MemoryId, Module, ValType,
};
use wasm_opt::{OptimizationOptions, ShrinkLevel};
use wasmtime_wasi::{pipe::MemoryInputPipe, WasiCtxBuilder};
use wizer::{Linker, Wizer};

use crate::{js_config::JsConfig, plugins::Plugin, JS};
use anyhow::Result;

static STDIN_PIPE: OnceLock<MemoryInputPipe> = OnceLock::new();

pub(crate) enum CodeGenType {
    /// Static code generation.
    Static,
    /// Dynamic code generation.
    Dynamic,
}

/// Identifiers used by the generated module.
// This is an internal detail of this module.
pub(crate) struct Identifiers {
    canonical_abi_realloc: FunctionId,
    eval_bytecode: Option<FunctionId>,
    invoke: FunctionId,
    memory: MemoryId,
}

impl Identifiers {
    fn new(
        canonical_abi_realloc: FunctionId,
        eval_bytecode: Option<FunctionId>,
        invoke: FunctionId,
        memory: MemoryId,
    ) -> Self {
        Self {
            canonical_abi_realloc,
            eval_bytecode,
            invoke,
            memory,
        }
    }
}

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

/// Code Generation.
pub(crate) struct Generator {
    /// Codegen type to use.
    pub ty: CodeGenType,
    /// JS runtime config.
    pub js_runtime_config: JsConfig,
    /// Plugin to use.
    pub plugin: Plugin,
    /// JavaScript function exports.
    function_exports: Exports,
    /// Whether to embed the compressed JS source in the generated module.
    pub source_compression: bool,
    /// WIT options for code generation.
    pub wit_opts: WitOptions,
}

impl Generator {
    /// Creates a new [`Generator`].
    pub fn new(ty: CodeGenType, js_runtime_config: JsConfig, plugin: Plugin) -> Self {
        Self {
            ty,
            js_runtime_config,
            source_compression: true,
            plugin,
            function_exports: Default::default(),
            wit_opts: Default::default(),
        }
    }

    /// Generate the starting module.
    fn generate_initial_module(&self) -> Result<Module> {
        let config = transform::module_config();
        let module = match &self.ty {
            CodeGenType::Static => {
                // Copy config JSON into stdin for `initialize_runtime` function.
                let runtime_config = self.js_runtime_config.to_json()?;
                STDIN_PIPE
                    .set(MemoryInputPipe::new(runtime_config))
                    .unwrap();
                let wasm = Wizer::new()
                    .init_func("initialize_runtime")
                    .make_linker(Some(Rc::new(move |engine| {
                        let mut linker = Linker::new(engine);
                        wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, move |cx| {
                            if cx.wasi_ctx.is_none() {
                                // The underlying buffer backing the pipe is an Arc
                                // so the cloning should be fast.
                                let config = STDIN_PIPE.get().unwrap().clone();
                                cx.wasi_ctx = Some(
                                    WasiCtxBuilder::new()
                                        .stdin(config)
                                        .inherit_stdout()
                                        .inherit_stderr()
                                        .build_p1(),
                                );
                            }
                            cx.wasi_ctx.as_mut().unwrap()
                        })?;
                        Ok(linker)
                    })))?
                    .wasm_bulk_memory(true)
                    .run(self.plugin.as_bytes())?;
                config.parse(&wasm)?
            }
            CodeGenType::Dynamic => Module::with_config(config),
        };
        Ok(module)
    }

    /// Resolve identifiers for functions and memory.
    pub fn resolve_identifiers(&self, module: &mut Module) -> Result<Identifiers> {
        match self.ty {
            CodeGenType::Static => {
                let canonical_abi_realloc_fn = module.exports.get_func("canonical_abi_realloc")?;
                let eval_bytecode = module.exports.get_func("eval_bytecode").ok();
                let invoke = module.exports.get_func("invoke")?;
                let ExportItem::Memory(memory) = module
                    .exports
                    .iter()
                    .find(|e| e.name == "memory")
                    .ok_or_else(|| anyhow::anyhow!("Missing memory export"))?
                    .item
                else {
                    anyhow::bail!("Export with name memory must be of type memory")
                };
                Ok(Identifiers::new(
                    canonical_abi_realloc_fn,
                    eval_bytecode,
                    invoke,
                    memory,
                ))
            }
            CodeGenType::Dynamic => {
                let import_namespace = self.plugin.import_namespace()?;
                let canonical_abi_realloc_type = module.types.add(
                    &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                    &[ValType::I32],
                );
                let (canonical_abi_realloc_fn_id, _) = module.add_import_func(
                    &import_namespace,
                    "canonical_abi_realloc",
                    canonical_abi_realloc_type,
                );

                // User plugins can use `invoke` with a null function name.
                // User plugins also won't have an `eval_bytecode` function to
                // import. We want to remove `eval_bytecode` from the default
                // plugin so we don't want to emit more uses of it.
                let eval_bytecode_fn_id = if self.plugin.is_v2_plugin() {
                    let eval_bytecode_type = module.types.add(&[ValType::I32, ValType::I32], &[]);
                    let (eval_bytecode_fn_id, _) = module.add_import_func(
                        &import_namespace,
                        "eval_bytecode",
                        eval_bytecode_type,
                    );
                    Some(eval_bytecode_fn_id)
                } else {
                    None
                };

                let invoke_type = module.types.add(
                    &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                    &[],
                );
                let (invoke_fn_id, _) =
                    module.add_import_func(&self.plugin.import_namespace()?, "invoke", invoke_type);

                let (memory_id, _) = module.add_import_memory(
                    &import_namespace,
                    "memory",
                    false,
                    false,
                    0,
                    None,
                    None,
                );

                Ok(Identifiers::new(
                    canonical_abi_realloc_fn_id,
                    eval_bytecode_fn_id,
                    invoke_fn_id,
                    memory_id,
                ))
            }
        }
    }

    /// Generate the main function.
    fn generate_main(
        &self,
        module: &mut Module,
        js: &JS,
        imports: &Identifiers,
    ) -> Result<BytecodeMetadata> {
        let bytecode = js.compile(&self.plugin)?;
        let bytecode_len: i32 = bytecode.len().try_into()?;
        let bytecode_data = module.data.add(DataKind::Passive, bytecode);

        let mut main = FunctionBuilder::new(&mut module.types, &[], &[]);
        let bytecode_ptr_local = module.locals.add(ValType::I32);
        let mut instructions = main.func_body();
        instructions
            // Allocate memory in plugin instance for bytecode array.
            .i32_const(0) // orig ptr
            .i32_const(0) // orig size
            .i32_const(1) // alignment
            .i32_const(bytecode_len) // new size
            .call(imports.canonical_abi_realloc)
            // Copy bytecode array into allocated memory.
            .local_tee(bytecode_ptr_local) // save returned address to local and set as dest addr for mem.init
            .i32_const(0) // offset into data segment for mem.init
            .i32_const(bytecode_len) // size to copy from data segment
            // top-2: dest addr, top-1: offset into source, top-0: size of memory region in bytes.
            .memory_init(imports.memory, bytecode_data);
        // Evaluate top level scope.
        if let Some(eval_bytecode) = imports.eval_bytecode {
            instructions
                .local_get(bytecode_ptr_local) // ptr to bytecode
                .i32_const(bytecode_len)
                .call(eval_bytecode);
        } else {
            // Assert we're not emitting a call with a null function to
            // invoke for the v2 plugin. `javy_quickjs_provider_v2` will never
            // support calling `invoke` with a null function. The default
            // plugin and user plugins do accept null functions.
            assert!(
                !self.plugin.is_v2_plugin(),
                "Using invoke with null function not supported for v2 plugin"
            );
            instructions
                .local_get(bytecode_ptr_local) // ptr to bytecode
                .i32_const(bytecode_len)
                .i32_const(0) // set function name ptr to null
                .i32_const(0) // set function name len to 0
                .call(imports.invoke);
        }
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
        identifiers: &Identifiers,
        bc_metadata: &BytecodeMetadata,
    ) -> Result<()> {
        if !self.function_exports.is_empty() {
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
                    .call(identifiers.canonical_abi_realloc)
                    .local_tee(bc_metadata.ptr)
                    .i32_const(0) // offset into data segment
                    .i32_const(bc_metadata.len) // size to copy
                    .memory_init(identifiers.memory, bc_metadata.data_section) // copy bytecode into allocated memory
                    .data_drop(bc_metadata.data_section)
                    // Copy function name.
                    .i32_const(0) // orig ptr
                    .i32_const(0) // orig len
                    .i32_const(1) // alignment
                    .i32_const(js_export_len) // new size
                    .call(identifiers.canonical_abi_realloc)
                    .local_tee(fn_name_ptr_local)
                    .i32_const(0) // offset into data segment
                    .i32_const(js_export_len) // size to copy
                    .memory_init(identifiers.memory, fn_name_data) // copy fn name into allocated memory
                    .data_drop(fn_name_data)
                    // Call invoke.
                    .local_get(bc_metadata.ptr)
                    .i32_const(bc_metadata.len)
                    .local_get(fn_name_ptr_local)
                    .i32_const(js_export_len)
                    .call(identifiers.invoke);
                let export_fn = export_fn.finish(vec![], &mut module.funcs);
                module.exports.add(&export.wit, export_fn);
            }
        }
        Ok(())
    }

    /// Clean-up the generated Wasm.
    fn postprocess(&self, module: &mut Module) -> Result<Vec<u8>> {
        match self.ty {
            CodeGenType::Static => {
                // Remove no longer necessary exports.
                module.exports.remove("canonical_abi_realloc")?;
                // User plugins won't have an `eval_bytecode` function that
                // Javy "owns".
                if !self.plugin.is_user_plugin() {
                    module.exports.remove("eval_bytecode")?;
                }
                module.exports.remove("invoke")?;
                module.exports.remove("compile_src")?;

                // Run wasm-opt to optimize.
                let tempdir = tempfile::tempdir()?;
                let tempfile_path = tempdir.path().join("temp.wasm");

                module.emit_wasm_file(&tempfile_path)?;

                OptimizationOptions::new_opt_level_3() // Aggressively optimize for speed.
                    .shrink_level(ShrinkLevel::Level0) // Don't optimize for size at the expense of performance.
                    .debug_info(false)
                    .run(&tempfile_path, &tempfile_path)?;

                Ok(fs::read(&tempfile_path)?)
            }
            CodeGenType::Dynamic => Ok(module.emit_wasm()),
        }
    }

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
    pub fn generate(&mut self, js: &JS) -> Result<Vec<u8>> {
        if self.wit_opts.defined() {
            self.function_exports = exports::process_exports(
                js,
                self.wit_opts.unwrap_path(),
                self.wit_opts.unwrap_world(),
            )?;
        }

        let mut module = self.generate_initial_module()?;
        let identifiers = self.resolve_identifiers(&mut module)?;
        let bc_metadata = self.generate_main(&mut module, js, &identifiers)?;
        self.generate_exports(&mut module, &identifiers, &bc_metadata)?;

        transform::add_producers_section(&mut module.producers);
        if !self.source_compression {
            module.customs.add(SourceCodeSection::uncompressed(js)?);
        } else {
            module.customs.add(SourceCodeSection::compressed(js)?);
        }

        let wasm = self.postprocess(&mut module)?;
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

#[cfg(test)]
mod test {
    use crate::js_config::JsConfig;
    use crate::plugins::Plugin;

    use super::Generator;
    use super::WitOptions;
    use anyhow::Result;

    #[test]
    fn default_values() -> Result<()> {
        let gen = Generator::new(
            crate::codegen::CodeGenType::Dynamic,
            JsConfig::default(),
            Plugin::Default,
        );
        assert!(gen.source_compression);
        assert!(matches!(gen.plugin, Plugin::Default));
        assert_eq!(gen.wit_opts, WitOptions::default());

        Ok(())
    }
}
