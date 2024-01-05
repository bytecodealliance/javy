use crate::{exports::Export, js::JS};

use super::transform::{self, SourceCodeSection};
use anyhow::Result;
use walrus::{DataKind, FunctionBuilder, Module, ValType};

// Run the calling code with the `dump_wat` feature enabled to print the WAT to stdout
//
// For the example generated WAT, the `bytecode_len` is 137
// (module
//    (type (;0;) (func))
//    (type (;1;) (func (param i32 i32)))
//    (type (;2;) (func (param i32 i32 i32 i32)))
//    (type (;3;) (func (param i32 i32 i32 i32) (result i32)))
//    (import "javy_quickjs_provider_v1" "canonical_abi_realloc" (func (;0;) (type 3)))
//    (import "javy_quickjs_provider_v1" "eval_bytecode" (func (;1;) (type 1)))
//    (import "javy_quickjs_provider_v1" "memory" (memory (;0;) 0))
//    (import "javy_quickjs_provider_v1" "invoke" (func (;2;) (type 2)))
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
pub fn generate(
    js: &JS,
    exported_functions: Vec<Export>,
    no_source_compression: bool,
) -> Result<Vec<u8>> {
    let mut module = Module::with_config(transform::module_config());

    const IMPORT_NAMESPACE: &str = "javy_quickjs_provider_v1";

    let canonical_abi_realloc_type = module.types.add(
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let (canonical_abi_realloc_fn, _) = module.add_import_func(
        IMPORT_NAMESPACE,
        "canonical_abi_realloc",
        canonical_abi_realloc_type,
    );

    let eval_bytecode_type = module.types.add(&[ValType::I32, ValType::I32], &[]);
    let (eval_bytecode_fn, _) =
        module.add_import_func(IMPORT_NAMESPACE, "eval_bytecode", eval_bytecode_type);

    let (memory, _) = module.add_import_memory(IMPORT_NAMESPACE, "memory", false, 0, None);

    transform::add_producers_section(&mut module.producers);
    if no_source_compression {
        module.customs.add(SourceCodeSection::uncompressed(js)?);
    } else {
        module.customs.add(SourceCodeSection::compressed(js)?);
    }

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
        .call(canonical_abi_realloc_fn)
        // Copy bytecode array into allocated memory.
        .local_tee(bytecode_ptr_local) // save returned address to local and set as dest addr for mem.init
        .i32_const(0) // offset into data segment for mem.init
        .i32_const(bytecode_len) // size to copy from data segment
        // top-2: dest addr, top-1: offset into source, top-0: size of memory region in bytes.
        .memory_init(memory, bytecode_data)
        // Evaluate bytecode.
        .local_get(bytecode_ptr_local) // ptr to bytecode
        .i32_const(bytecode_len)
        .call(eval_bytecode_fn);
    let main = main.finish(vec![], &mut module.funcs);

    module.exports.add("_start", main);

    if !exported_functions.is_empty() {
        let invoke_type = module.types.add(
            &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            &[],
        );
        let (invoke_fn, _) = module.add_import_func(IMPORT_NAMESPACE, "invoke", invoke_type);

        let fn_name_ptr_local = module.locals.add(ValType::I32);
        for export in exported_functions {
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
                .i32_const(bytecode_len) // size to copy
                .call(canonical_abi_realloc_fn)
                .local_tee(bytecode_ptr_local)
                .i32_const(0) // offset into data segment
                .i32_const(bytecode_len) // size to copy
                .memory_init(memory, bytecode_data) // copy bytecode into allocated memory
                .data_drop(bytecode_data)
                // Copy function name.
                .i32_const(0) // orig ptr
                .i32_const(0) // orig len
                .i32_const(1) // alignment
                .i32_const(js_export_len) // new size
                .call(canonical_abi_realloc_fn)
                .local_tee(fn_name_ptr_local)
                .i32_const(0) // offset into data segment
                .i32_const(js_export_len) // size to copy
                .memory_init(memory, fn_name_data) // copy fn name into allocated memory
                .data_drop(fn_name_data)
                // Call invoke.
                .local_get(bytecode_ptr_local)
                .i32_const(bytecode_len)
                .local_get(fn_name_ptr_local)
                .i32_const(js_export_len)
                .call(invoke_fn);
            let export_fn = export_fn.finish(vec![], &mut module.funcs);
            module.exports.add(&export.wit, export_fn);
        }
    }

    let wasm = module.emit_wasm();
    print_wat(&wasm)?;
    Ok(wasm)
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
