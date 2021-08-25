#include "../quickjs/quickjs.h"

JSValue JS_NewBool_Ext(JSContext *ctx, JS_BOOL val) {
  return JS_MKVAL(JS_TAG_BOOL, (val != 0));
}

JSValue JS_NewInt32_Ext(JSContext *ctx, int32_t val) {
  return JS_NewInt32(ctx, val);
}

JSValue JS_NewUint32_Ext(JSContext *ctx, uint32_t val) {
  return JS_NewUint32(ctx, val);
}


JSValue JS_NewFloat64_Ext(JSContext *ctx, double d) {
  return JS_NewFloat64(ctx, d);
}

JS_BOOL JS_IsFloat64_Ext(int tag) {
  return JS_TAG_IS_FLOAT64(tag);
}

const JSValue ext_js_null = JS_NULL;
const JSValue ext_js_undefined = JS_UNDEFINED;
const JSValue ext_js_false = JS_FALSE;
const JSValue ext_js_true = JS_TRUE;
const JSValue ext_js_exception = JS_EXCEPTION;
const JSValue ext_js_uninitialized = JS_UNINITIALIZED;

