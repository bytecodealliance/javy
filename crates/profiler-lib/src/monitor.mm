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

// Switching the dispatch target closes out the opcode that just ran
// and begins the new one.
// Declared before the increment probe so this `br_table` counts toward
// current JS opcode it dispatches to, not the previous one.
wasm:opcode:br_table:before / @static {{LIBRARY_NAME}}.is_dispatch_func(fid as i32) as bool / {
    {{LIBRARY_NAME}}.set_dispatch_target(target as i32, instruction_count);
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

wasm:func:exit / fname == "_start" / {
    // At program exit emit the single, whole-execution report.
    {{LIBRARY_NAME}}.report();
}
