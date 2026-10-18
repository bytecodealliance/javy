use {{LIBRARY_NAME}};

// Global count of executed countable Wasm instructions, across every
// function.
var instruction_count: i64;

wasm:func:entry / @static {{LIBRARY_NAME}}.is_dispatch_func(fid as i32) as bool / {
    // Push a new JS function frame.
    {{LIBRARY_NAME}}.start_func();
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

// The dispatch load reads the next QuickJS opcode, and runs once per
// dispatch. It closes out the site that just ran and begins the new one:
// - its result is the opcode, rather than the dispatch `br_table` index,
//   which the compiler is free to alter depending on the structure of the
//   `br_table`;
// - its effective address is where the opcode was read, recorded as an
//   absolute address. Which function it belongs to is resolved after the
//   run; see `Profiler` in `state.rs`.
// The load itself is counted toward the previous site, the instructions
// from here to the `br_table` toward the new one.
// The load kinds matched here must be those `interpreter::is_byte_load`
// accepts.
wasm:opcode:i32.load8_u|i32.load8_s(res0: i32):after / @static {{LIBRARY_NAME}}.is_dispatch_load(fid as i32, pc as i32) as bool / {
    {{LIBRARY_NAME}}.dispatch(res0, effective_addr as i32, instruction_count);
}
