use anyhow::Result;
use wasm_encoder::{
    CodeSection, CustomSection, DataCountSection, DataSection, EntityType, ExportKind,
    ExportSection, Function, FunctionSection, ImportSection, Instruction, MemoryType, Module,
    TypeSection, ValType,
};

use crate::source_code_section;

// Run the calling code with the `dump_wat` feature enabled to print the WAT to stdout
//
// For the example generated WAT, the `bytecode_len` is 145
// (module
//     (type (;0;) (func (param i32 i32 i32 i32) (result i32)))
//     (type (;1;) (func (param i32 i32)))
//     (type (;2;) (func))
//     (import "javy_quickjs_provider_v1" "canonical_abi_realloc" (func (;0;) (type 0)))
//     (import "javy_quickjs_provider_v1" "eval_bytecode" (func (;1;) (type 1)))
//     (import "javy_quickjs_provider_v1" "memory" (memory (;0;) 0))
//     (func (;2;) (type 2)
//       (local i32)
//       i32.const 0
//       i32.const 0
//       i32.const 1
//       i32.const 145
//       call 0
//       local.tee 0
//       i32.const 0
//       i32.const 145
//       memory.init 0
//       data.drop 0
//       local.get 0
//       i32.const 145
//       call 1
//     )
//     (export "_start" (func 2))
//     (data (;0;) "\02\08\0econsole\06log\16TextDecoder\0cdecode\16TextEncoder\0cencode\03\00\d8\18function.mjs\0e\00\06\00\a0\01\00\01\00\07\00\006\01\a2\01\00\00\008\de\00\00\00B\df\00\00\008\e0\00\00\00\11!\00\00B\e1\00\00\008\e2\00\00\00\11!\00\00B\e3\00\00\00\04\e4\00\00\00$\01\00$\01\00$\01\00\cd(\ca\03\01\00")
// )
pub fn generate_module(bytecode: Vec<u8>, js_src: &[u8]) -> Result<Vec<u8>> {
    let mut module = Module::new();
    let mut indices = Indices::new();

    add_types(&mut module, &mut indices);
    add_imports(&mut module, &mut indices);
    add_functions(&mut module, &mut indices);
    add_exports(&mut module, &indices);
    add_data_count(&mut module, 1);
    add_code(&mut module, &indices, bytecode.len().try_into()?);
    add_data(&mut module, bytecode);
    add_source_code(&mut module, js_src)?;

    let wasm_binary = module.finish();
    print_wat(&wasm_binary)?;
    Ok(wasm_binary)
}

struct Indices {
    pub realloc_ty: Option<u32>,
    pub eval_bytecode_ty: Option<u32>,
    pub start_ty: Option<u32>,
    pub realloc_fn: Option<u32>,
    pub eval_bytecode_fn: Option<u32>,
    pub start_fn: Option<u32>,
    pub javy_quickjs_provider_memory: Option<u32>,
    pub bytecode_data: u32,
    next_ty_index: u32,
    next_func_index: u32,
    next_memory_index: u32,
}

impl Indices {
    pub fn new() -> Indices {
        Indices {
            realloc_ty: None,
            eval_bytecode_ty: None,
            start_ty: None,
            realloc_fn: None,
            eval_bytecode_fn: None,
            start_fn: None,
            javy_quickjs_provider_memory: None,
            bytecode_data: 0,
            next_ty_index: 0,
            next_func_index: 0,
            next_memory_index: 0,
        }
    }

    pub fn assign_realloc_ty(&mut self) {
        self.realloc_ty = Some(self.next_ty_index);
        self.next_ty_index += 1;
    }

    pub fn assign_eval_bytecode_ty(&mut self) {
        self.eval_bytecode_ty = Some(self.next_ty_index);
        self.next_ty_index += 1;
    }

    pub fn assign_start_ty(&mut self) {
        self.start_ty = Some(self.next_ty_index);
        self.next_ty_index += 1;
    }

    pub fn assign_realloc_fn(&mut self) {
        self.realloc_fn = Some(self.next_func_index);
        self.next_func_index += 1;
    }

    pub fn assign_eval_bytecode_fn(&mut self) {
        self.eval_bytecode_fn = Some(self.next_func_index);
        self.next_func_index += 1;
    }

    pub fn assign_start_fn(&mut self) {
        self.start_fn = Some(self.next_func_index);
        self.next_func_index += 1;
    }

    pub fn assign_javy_quickjs_provider_memory(&mut self) {
        self.javy_quickjs_provider_memory = Some(self.next_memory_index);
        self.next_memory_index += 1;
    }
}

fn add_types(module: &mut Module, indices: &mut Indices) {
    let mut types = TypeSection::new();

    // canonical_abi_realloc
    types.function(
        vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        vec![ValType::I32],
    );
    indices.assign_realloc_ty();

    // eval_bytecode
    types.function(vec![ValType::I32, ValType::I32], vec![]);
    indices.assign_eval_bytecode_ty();

    // _start
    types.function(vec![], vec![]);
    indices.assign_start_ty();

    module.section(&types);
}

fn add_imports(module: &mut Module, indices: &mut Indices) {
    const IMPORT_NAMESPACE: &str = "javy_quickjs_provider_v1";
    let mut imports = ImportSection::new();

    imports.import(
        IMPORT_NAMESPACE,
        "canonical_abi_realloc",
        EntityType::Function(indices.realloc_ty.unwrap()),
    );
    indices.assign_realloc_fn();

    imports.import(
        IMPORT_NAMESPACE,
        "eval_bytecode",
        EntityType::Function(indices.eval_bytecode_ty.unwrap()),
    );
    indices.assign_eval_bytecode_fn();

    imports.import(
        IMPORT_NAMESPACE,
        "memory",
        EntityType::Memory(MemoryType {
            minimum: 0,
            maximum: None,
            memory64: false,
            shared: false,
        }),
    );
    indices.assign_javy_quickjs_provider_memory();

    module.section(&imports);
}

fn add_functions(module: &mut Module, indices: &mut Indices) {
    let mut functions = FunctionSection::new();
    functions.function(indices.start_ty.unwrap());
    indices.assign_start_fn();
    module.section(&functions);
}

fn add_exports(module: &mut Module, indices: &Indices) {
    let mut exports = ExportSection::new();
    exports.export("_start", ExportKind::Func, indices.start_fn.unwrap());
    module.section(&exports);
}

fn add_data_count(module: &mut Module, count: u32) {
    module.section(&DataCountSection { count });
}

fn add_code(module: &mut Module, indices: &Indices, bytecode_len: i32) {
    let mut code = CodeSection::new();

    let mut start_function = Function::new_with_locals_types([ValType::I32]);
    const ALLOCATED_PTR_LOCAL_INDEX: u32 = 0;

    // allocate memory in javy_quickjs_provider for bytecode array
    start_function.instruction(&Instruction::I32Const(0)); // orig ptr
    start_function.instruction(&Instruction::I32Const(0)); // orig size
    start_function.instruction(&Instruction::I32Const(1)); // alignment
    start_function.instruction(&Instruction::I32Const(bytecode_len)); // new_size
    start_function.instruction(&Instruction::Call(indices.realloc_fn.unwrap()));

    // copy bytecode array into allocated memory
    start_function.instruction(&Instruction::LocalTee(ALLOCATED_PTR_LOCAL_INDEX)); // set local to allocated ptr, also sets allocated ptr as dest addr for mem init
    start_function.instruction(&Instruction::I32Const(0)); // offset into data segment
    start_function.instruction(&Instruction::I32Const(bytecode_len)); // size to copy from data segment

    // top-2: dest addr, top-1: offset into source, top-0: size of memory region in bytes
    start_function.instruction(&Instruction::MemoryInit {
        mem: indices.javy_quickjs_provider_memory.unwrap(),
        data_index: indices.bytecode_data,
    });
    start_function.instruction(&Instruction::DataDrop(indices.bytecode_data)); // no longer need data section so reduce memory pressure

    // evaluate bytecode
    start_function.instruction(&Instruction::LocalGet(ALLOCATED_PTR_LOCAL_INDEX)); // bytecode_ptr
    start_function.instruction(&Instruction::I32Const(bytecode_len));
    start_function.instruction(&Instruction::Call(indices.eval_bytecode_fn.unwrap())); // eval_bytecode
    start_function.instruction(&Instruction::End);

    code.function(&start_function);
    module.section(&code);
}

fn add_data(module: &mut Module, bytecode: Vec<u8>) {
    let mut data = DataSection::new();
    data.passive(bytecode);
    module.section(&data);
}

fn add_source_code(module: &mut Module, js_src: &[u8]) -> Result<()> {
    let compressed_source_code = source_code_section::compress_source_code(js_src)?;
    let source_code_custom = CustomSection {
        name: source_code_section::SOURCE_CODE_SECTION_NAME,
        data: &compressed_source_code,
    };
    module.section(&source_code_custom);
    Ok(())
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
