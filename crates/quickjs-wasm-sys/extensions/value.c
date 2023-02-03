#include "../quickjs/quickjs.h"
#include "../quickjs/libbf.h"

JSValue JS_NewBool_Ext(JSContext *ctx, JS_BOOL val) {
  return JS_MKVAL(JS_TAG_BOOL, (val != 0));
}

JSValue JS_NewInt32_Ext(JSContext *ctx, int32_t val) {
  return JS_NewInt32(ctx, val);
}

JSValue JS_NewUint32_Ext(JSContext *ctx, uint32_t val) {
  return JS_NewUint32(ctx, val);
}

JSValue JS_NewInt64_Ext(JSContext *ctx, int64_t val) {
  return JS_NewInt64(ctx, val);
}

JSValue JS_NewFloat64_Ext(JSContext *ctx, double d) {
  return JS_NewFloat64(ctx, d);
}

JS_BOOL JS_IsFloat64_Ext(int tag) {
  return JS_TAG_IS_FLOAT64(tag);
}

JS_BOOL JS_IsArrayBuffer_Ext(JSContext* ctx, JSValue val) {
    size_t len;
    return JS_GetArrayBuffer(ctx, &len, val) != 0;
}

typedef struct JSBigFloat {
    JSRefCountHeader header; /* must come first, 32-bit */
    bf_t num;
} JSBigFloat;

JS_BOOL JS_BigIntSigned(JSContext *ctx, JSValue val) {
  JSBigFloat *p = JS_VALUE_GET_PTR(val);
  return p->num.sign;
}

// creates a copy of the value and returns it as an int64_t
static int JS_BigIntToInt64Free(JSContext *ctx, int64_t *pres, JSValue val) {
  JSBigFloat *p = JS_VALUE_GET_PTR(val);
  if (bf_get_int64(pres, &p->num, 0) != 0) {
    JS_FreeValue(ctx, val);
    return -1;
  }

  JS_FreeValue(ctx, val);
  return 0;
}

int JS_BigIntToInt64(JSContext *ctx, int64_t *pres, JSValueConst val) {
  return JS_BigIntToInt64Free(ctx, pres, JS_DupValue(ctx, val));
}

// creates a copy of the value and returns it as an uint64_t
static int JS_BigIntToUint64Free(JSContext *ctx, uint64_t *pres, JSValue val) {
  JSBigFloat *p = JS_VALUE_GET_PTR(val);
  if (bf_get_uint64(pres, &p->num) != 0) {
    JS_FreeValue(ctx, val);
    return -1;
  }

  JS_FreeValue(ctx, val);
  return 0;
}

int JS_BigIntToUint64(JSContext *ctx, uint64_t *pres, JSValueConst val) {
  return JS_BigIntToUint64Free(ctx, pres, JS_DupValue(ctx, val));
}


const JSValue ext_js_null = JS_NULL;
const JSValue ext_js_undefined = JS_UNDEFINED;
const JSValue ext_js_false = JS_FALSE;
const JSValue ext_js_true = JS_TRUE;
const JSValue ext_js_uninitialized = JS_UNINITIALIZED;
