use anyhow::Result;
use walrus::{DataKind, FunctionBuilder, Module, ValType};

use crate::{
    js::JS,
    transform::{self, SourceCodeSection},
};

// Run the calling code with the `dump_wat` feature enabled to print the WAT to stdout
//
// For the example generated WAT, the `bytecode_len` is 90
// (module
//     (type (;0;) (func))
//     (type (;1;) (func (param i32 i32)))
//     (type (;2;) (func (param i32 i32 i32 i32) (result i32)))
//     (import "javy_quickjs_provider_v1" "canonical_abi_realloc" (func (;0;) (type 2)))
//     (import "javy_quickjs_provider_v1" "eval_bytecode" (func (;1;) (type 1)))
//     (import "javy_quickjs_provider_v1" "memory" (memory (;0;) 0))
//     (func (;2;) (type 0)
//       (local i32)
//       i32.const 0
//       i32.const 0
//       i32.const 1
//       i32.const 90
//       call 0
//       local.tee 0
//       i32.const 0
//       i32.const 90
//       memory.init 0
//       local.get 0
//       i32.const 90
//       call 1
//     )
//     (export "_start" (func 2))
//     (data (;0;) "\02\04\18function.mjs\0econsole\06log\18Hello world!\0f\bc\03\00\00\00\00\0e\00\06\01\a0\01\00\00\00\03\00\00\17\00\08\ea\02)8\df\00\00\00B\e0\00\00\00\04\e1\00\00\00$\01\00)\bc\03\01\02\01\17")
// )
pub fn generate_module(js: &JS) -> Result<Vec<u8>> {
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
    module.customs.add(SourceCodeSection::new(js)?);

    let bytecode = js.compile()?;
    let bytecode_len: i32 = bytecode.len().try_into()?;
    let bytecode_data = module.data.add(DataKind::Passive, bytecode);

    let mut main = FunctionBuilder::new(&mut module.types, &[], &[]);
    let ptr_local = module.locals.add(ValType::I32);
    main.func_body()
        // Allocate memory in javy_quickjs_provider for bytecode array.
        .i32_const(0) // orig ptr
        .i32_const(0) // orig size
        .i32_const(1) // alignment
        .i32_const(bytecode_len) // new size
        .call(canonical_abi_realloc_fn)
        // Copy bytecode array into allocated memory.
        .local_tee(ptr_local) // save returned address to local and set as dest addr for mem.init
        .i32_const(0) // offset into data segment for mem.init
        .i32_const(bytecode_len) // size to copy from data segment
        // top-2: dest addr, top-1: offset into source, top-0: size of memory region in bytes.
        .memory_init(memory, bytecode_data)
        // Evaluate bytecode.
        .local_get(ptr_local) // ptr to bytecode
        .i32_const(bytecode_len)
        .call(eval_bytecode_fn);
    let main = main.finish(vec![], &mut module.funcs);

    module.exports.add("_start", main);

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
