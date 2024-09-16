# JavaScript API Support

Javy by default supports ES2023, plus partial support for additional APIs.
NodeJS APIs are not supported.

This document aims to give an overview of the additional APIs provided by Javy.

In general the ultimate goal of Javy is to provide a [WinterCG  Common
API](https://common-min-api.proposal.wintercg.org/#api-index) compatible
implementation, however, currently some APIs are not fully
compliant and therefore are provided under a custom `Javy` namespace or
explicitly marked as partially supported in the table below.

## Compatibility table

|API|Support|Comments|
|:-:|:-:|:-:|
|`JSON`|✅| Improved performace through SIMD JSON, when using the `-J simd-json-builtins` flag|
|`TexDecoder`|🚧| Partial support, not fully compliant|
|`TextEncoder`|🚧| Partial support, not fully compliant|
|`TextEncoder`|🚧| Partial support, not fully compliant|
|`console`|🚧| Partial support, `console.log` and `console.error`|

Javy provides a custom `Javy` namespace, which includes the following
functionality:

* `IO`: provides `readSync` and `writeSync`, analogous to [Node's `fs`
  API](https://nodejs.org/api/fs.html).

* `JSON`: provides `fromStdin()` and `toStdout()`. Which are helpers to read or
  write from and to a file descriptor when working with `JSON`.

