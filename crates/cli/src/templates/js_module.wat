(module
  (import "js_engine" "core_malloc" (func $core_malloc (param i32) (result i32)))
  (import "js_engine" "run_js_script" (func $run_js_script (param i32 i32)))
  (import "js_engine" "memory" (memory $js_engine_memory 1)) ;; 0
  (memory $js_code_memory 1) ;; 1

  (func (export "shopify_main")
    (local $malloc_result i32)
    (local $js_string_length_bytes i32)
    (local.tee $js_string_length_bytes (i32.const {{ js_string_length_bytes }}))

    (call $core_malloc)
    (local.tee $malloc_result) ;; Destination address (result of malloc)

    ;; Address of JS in our memory
    i32.const 1 
    local.get $js_string_length_bytes ;; length of the copy
    memory.copy $js_engine_memory $js_code_memory ;; Copy to imported (0) from our memory (1)

    local.get $malloc_result
    local.get $js_string_length_bytes
    call $run_js_script
  )

  ;; This data segments initializes memory 1
  ;; which is JavaScript's module memory
  ;; at offset 1
  (data 1 (i32.const 1) "{{ js_string_bytes }}")
)
