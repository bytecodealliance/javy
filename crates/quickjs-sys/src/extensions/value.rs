extern "C" {
    pub fn JS_NewBool_Ext(ctx: *mut JSContext, bool: i32) -> JSValue;
    pub fn JS_NewFloat64_Ext(ctx: *mut JSContext, float: f64) -> JSValue;
    pub fn JS_IsFloat64_Ext(tag: i32) -> i32;
}
