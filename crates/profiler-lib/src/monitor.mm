use {{LIBRARY_NAME}};

// Global count of executed countable Wasm instructions, across every
// function.
var instruction_count: i64;

// Set to true on the dispatch function entry to capture the function's
// address from its first dispatch load.
var expect_first_load: bool;

wasm:func:entry / @static {{LIBRARY_NAME}}.is_dispatch_func(fid as i32) as bool / {
    // Push a new JS function frame.
    {{LIBRARY_NAME}}.start_func();
    expect_first_load = true;
}

wasm:func:exit / @static {{LIBRARY_NAME}}.is_dispatch_func(fid as i32) as bool / {
    // Pop the topmost JS function frame. The outermost activation closes
    // out the final opcode with the current instruction count.
    {{LIBRARY_NAME}}.exit_func(instruction_count);
}

// Increment the instruction count, iff the opcode is countable.
wasm:opcode:*:before / @static {{LIBRARY_NAME}}.is_countable_opcode(fid as i32, pc as i32) as bool / {
    instruction_count = instruction_count + 1;
}

wasm:opcode:*load*:before / expect_first_load && @static {{LIBRARY_NAME}}.is_dispatch_load(fid as i32, pc as i32) as bool / {
    // First load of the frame: its effective address identifies the JS
    // function.
    {{LIBRARY_NAME}}.set_func_addr(effective_addr as i32);
    expect_first_load = false;
}

// The dispatch load reads the next QuickJS opcode, and runs once per
// dispatch. Its result closes out the opcode that just ran and begins the
// new one. The opcode is the loaded byte rather than the dispatch
// `br_table` index, which the compiler is free alter depending on the structure
// of the `br_table`.
// Being `:after`, it runs after the `:before` first-load probe, so the
// frame's function address is known by the time the first opcode is
// attributed. The load itself is
// counted toward the previous opcode, the instructions from here to the
// `br_table` toward the new one.
// The load kinds matched here must be those `interpreter::is_byte_load`
// accepts.
wasm:opcode:i32.load8_u|i32.load8_s(res0: i32):after / @static {{LIBRARY_NAME}}.is_dispatch_load(fid as i32, pc as i32) as bool / {
    {{LIBRARY_NAME}}.set_opcode(res0, instruction_count);
}
