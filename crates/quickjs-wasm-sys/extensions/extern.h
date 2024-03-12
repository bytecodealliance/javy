#include "../quickjs/quickjs.h"

JSValue JS_NewBool_Ext(JSContext *ctx, JS_BOOL val);
JSValue JS_NewInt32_Ext(JSContext *ctx, int32_t val);
JSValue JS_NewUint32_Ext(JSContext *ctx, uint32_t val);
JSValue JS_NewInt64_Ext(JSContext *ctx, int64_t val);
JSValue JS_NewFloat64_Ext(JSContext *ctx, double d);
JS_BOOL JS_IsFloat64_Ext(int tag);
JS_BOOL JS_IsArrayBuffer_Ext(JSContext *ctx, JSValue val);
JS_BOOL JS_BigIntSigned(JSContext *ctx, JSValue val);
int JS_BigIntToInt64(JSContext *ctx, int64_t *pres, JSValueConst val);
int JS_BigIntToUint64(JSContext *ctx, uint64_t *pres, JSValueConst val);

void JS_FreeValue_Ext(JSContext *ctx, JSValue v);
JSValue JS_DupValue_Ext(JSContext *ctx, JSValueConst v);

const JSValue ext_js_null;
const JSValue ext_js_undefined;
const JSValue ext_js_false;
const JSValue ext_js_true;
const JSValue ext_js_uninitialized;
