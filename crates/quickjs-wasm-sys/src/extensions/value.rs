extern "C" {
    pub fn JS_NewBool_Ext(ctx: *mut JSContext, bool: i32) -> JSValue;
    pub fn JS_NewInt32_Ext(ctx: *mut JSContext, val: i32) -> JSValue;
    pub fn JS_NewUint32_Ext(ctx: *mut JSContext, val: u32) -> JSValue;
    pub fn JS_NewInt64_Ext(ctx: *mut JSContext, val: i64) -> JSValue;
    pub fn JS_NewFloat64_Ext(ctx: *mut JSContext, float: f64) -> JSValue;
    pub fn JS_IsFloat64_Ext(tag: i32) -> i32;
    pub fn JS_IsArrayBuffer_Ext(ctx: *mut JSContext, value: JSValue) -> i32;
    pub fn JS_BigIntSigned(ctx: *mut JSContext, val: JSValue) -> i32;
    pub fn JS_BigIntToInt64(ctx: *mut JSContext, plen: *mut i64, val: JSValue) -> i32;
    pub fn JS_BigIntToUint64(ctx: *mut JSContext, plen: *mut u64, val: JSValue) -> i32;
    pub static ext_js_null: JSValue;
    pub static ext_js_undefined: JSValue;
    pub static ext_js_false: JSValue;
    pub static ext_js_true: JSValue;
    pub static ext_js_uninitialized: JSValue;
}
