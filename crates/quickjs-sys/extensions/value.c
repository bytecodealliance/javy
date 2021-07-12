#include "../quickjs/quickjs.h"

JSValue JS_NewBool_Ext(JSContext *ctx, JS_BOOL val) {
  return JS_MKVAL(JS_TAG_BOOL, (val != 0));
}

JSValue JS_NewFloat64_Ext(JSContext *ctx, double d) {
  return JS_NewFloat64(ctx, d);
}

JS_BOOL JS_IsFloat64_Ext(int tag) {
  return JS_TAG_IS_FLOAT64(tag);
}
