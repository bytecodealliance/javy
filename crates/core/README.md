# msgpack <-> js value

Here's a table that maps the [msgpack specification](https://github.com/msgpack/msgpack/blob/master/spec.md) to the different datatypes in Javascript.

|msgpack|js value|
|-|-|
|positive fixint|number|
|fixmap|js object|
|fixarray|js array|
|fixstr|js string|
|nil|null|
|false|false|
|true|true|
|bin 8|not supported|
|bin 16|not supported|
|bin 32|not supported|
|float 32|number|
|float 64|not supported|
|uint 8|number|
|uint 16|number|
|uint 32|number|
|uint 64|bigint|
|int 8|number|
|int 16|number|
|int 32|number|
|int 64|bigint|
|str 8|js string|
|str 16|js string|
|str 32|js string|
|map 16|js object|
|map 32|js object|
|negative fixint|number|

# number roundtrips

Javascript numbers are an absolute nightmare. Here's what a full roundtrip looks like for a set of different number values to help understand the different conversions.

TODO...

|v|ruby|ruby-msgpack|rmp_serde|js|rmp_serde|ruby-msgpack|ruby|
|-|-|-|-|-|-|-|-|
|0||||||||
|i8::MIN||||||||
|i8::MAX||||||||
|i16::MIN||||||||
|i16::MAX||||||||
|i32::MIN||||||||
|i32::MAX||||||||
|i64::MIN||||||||
|i64::MAX||||||||
|u8::MAX||||||||
|u16::MAX||||||||
|u32::MAX||||||||
|u64::MAX||||||||
|f32::MIN||||||||
|f32::MAX||||||||
|f32::NAN||||||||
|f32::INFINITY||||||||
|f32::NEG_INFINITY||||||||
|-0.0||||||||
|0.0||||||||

# number vs bigint

Javascript originally didn't support i64/u64 values. In order to support theses values, [BigInt was added to the specification](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt) and is nowadays supported by most web browsers. However BigInt can't naturally interact with other number types, they need to be converted explicitely to a BigInt.

Since the current serialization process is schemaless, it is currently impossible to know during the transcoding process when to serialize a small value (e.g. `42`) as a i64/u64, or as a BigInt.

For a more complete example, the following script would fail:

```javascript
var Shopify;
Shopify = {
  main: (productId) => {
    // The serializer does not know that the value must be serialized as a BigInt.
    if (i === BigInt(42)) {
      return "ok";
    }

    throw new Error("i is not BigInt(42)");
  }
}
```

In order to make the following script succeed, `productId` would need to be explicitely converted to a `BigInt` at runtime. From a **typescript** perspective, the interface of the script in this scenario would be the following:

```typescript
interface Input {
  productId: number | bigint
}
```

# notes

- rmp_serde serializes i64 that are [larger than an i32 as a u64](https://github.com/3Hren/msgpack-rust/blob/aa3c4a77b2b901fe73a555c615b92773b40905fc/rmp/src/encode/sint.rs#L170).
- rmp_serde serializes to i64 only when values are < i32::MIN.
- rmp_serde always tries to serializes to the smallest format. See previous section...
- QuickJS optimises u32 values < i32::MAX and [store them as integer instead of floats](https://github.com/shopify/javy/blob/e00c5efad4abe2a4288517017d46db24ff862e7e/crates/quickjs-sys/quickjs/quickjs.h#L531-L540)
- QuickJS also optimizes f32, and f64
- We do not use QuickJS [NewInt64](https://github.com/shopify/javy/blob/e00c5efad4abe2a4288517017d46db24ff862e7e/crates/quickjs-sys/quickjs/quickjs.h#L520-L529), we instead directly create a BigInt value.
- BigInts are not clamped to MIN/MAX, they instead return an error when they overflow/underflow.
