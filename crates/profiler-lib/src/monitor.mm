use {{LIBRARY_NAME}};

// Ensure that we only read the first address of the function
// bytecode, which serves as the function identifier.
var expect_first_load: bool;

// True only while executing a handler body, that is the window
// between a dispatch `br_table` and the next dispatch load. Provides
// more accurate counting of Wasm-instructions-per-JS-opcode.
var counting: bool;

wasm:func:entry / @static {{LIBRARY_NAME}}.is_dispatch_func(fid as i32) as bool / {
    // When entering the dispatch function, track a new JS function frame.
    {{LIBRARY_NAME}}.start_func();
    // When entering the dispatch function, we expect the first load later on.
    expect_first_load = true;
}

wasm:func:exit / @static {{LIBRARY_NAME}}.is_dispatch_func(fid as i32) as bool / {
    // When exiting the dispatch function, pop the current JS function frame.
    {{LIBRARY_NAME}}.exit_func();
}

wasm:opcode:br_table:before / @static {{LIBRARY_NAME}}.is_dispatch_func(fid as i32) as bool / {
    // We are in the dispatch loop, so we track the current dispatch target.
    {{LIBRARY_NAME}}.set_dispatch_target(target as i32);
    // Entering a handler body for `target`, so we begin counting.
    counting = true;
}

wasm:opcode:*load*:before / @static {{LIBRARY_NAME}}.is_dispatch_load(fid as i32, pc as i32) as bool / {
    // When hitting the dispatch load, stop counting, the current and
    // following instructions do not belong to any JS opcode handler.
    counting = false;
}

wasm:opcode:*load*:before / expect_first_load && @static {{LIBRARY_NAME}}.is_dispatch_load(fid as i32, pc as i32) as bool / {
    // Set the current function identifier, which is the effective address of the dispatch load.
    // We no longer care about `expect_first_load`, this probe should
    // only fire once per JS function invocation.
    {{LIBRARY_NAME}}.set_func_addr(effective_addr as i32);
    expect_first_load = false;
}

wasm:opcode:*:before / @static {{LIBRARY_NAME}}.is_dispatch_func(fid as i32) as bool
    && counting
    && opname != "nop"
    && opname != "drop"
    && opname != "block"
    && opname != "loop"
    && opname != "unreachable"
    && opname != "return"
    && opname != "else"
    && opname != "end" / {

    {{LIBRARY_NAME}}.handle_opcode(pc as i32);
}
