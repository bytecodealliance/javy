#[unsafe(no_mangle)]
pub extern "C" fn canonical_abi_realloc(
    _old_ptr: u32,
    _old_len: u32,
    _alignment: u32,
    _new_len: u32,
) -> u32 {
    unimplemented!()
}

#[unsafe(no_mangle)]
pub extern "C" fn compile_src(_src_ptr: u32, _src_len: u32) -> u32 {
    unimplemented!()
}

#[unsafe(no_mangle)]
pub extern "C" fn invoke(
    _bytecode_ptr: u32,
    _bytecode_len: u32,
    _fn_name_ptr: u32,
    _fn_name_len: u32,
) {
    unimplemented!()
}
