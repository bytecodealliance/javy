export default [
  {
    testFile: "./custom_test.js",
    ignoredTests: ["This is an ignored test"],
  },
  {
    testFile: "upstream/encoding/api-basics.any.js",
    ignoredTests: ["Decode sample: utf-16le", "Decode sample: utf-16be", "Decode sample: utf-16"],
  },
  // { // FIXME script importing isn't working
  //   testFile: "upstream/encoding/api-invalid-label.any.js",
  // },
  // { // FIXME script importing isn't working
  //   testFile: "upstream/encoding/api-replacement-encodings.any.js",
  // },
  // { // FIXME needs fix for TextEncoder to be merged
  //   testFile: "upstream/encoding/api-surrogates-utf8.any.js",
  // },
  // { // FIXME requires `encodeInto` support
  //   testFile: "upstream/encoding/encodeInto.any.js",
  // },
  // { // FIXME script importing isn't working
  //   testFile: "upstream/encoding/replacement-encodings.any.js",
  // },
  // { // FIXME need to add streaming support
  //   testFile: "upstream/encoding/textdecoder-arguments.any.js",
  // },
  // { // FIXME need to fix failing BOM test
  //   testFile: "upstream/encoding/textdecoder-byte-order-marks.any.js",
  //   ignoredTests: ["Byte-order marks: utf-16le", "Byte-order marks: utf-16be"],
  // },
  {
    testFile: "upstream/encoding/textdecoder-eof.any.js",
    ignoredTests: ["/stream: true/"],
  },
  // { // FIXME need to fix the type of the exception thrown
  //   testFile: "custom_tests/textdecoder-fatal-streaming.any.js",
  // },
  // { // FIXME need to fix the type of the exception thrown when fatal is set to `true`
  //   testFile: "upstream/encoding/textdecoder-fatal.any.js",
  //   ignoredTests: ["Fatal flag: utf-16le - truncated code unit"],
  // },
  // { // FIXME need to fix failing BOM test
  //   testFile: "upstream/encoding/textdecoder-ignorebom.any.js",
  //   ignoredTests: ["/utf-16/"]
  // },
  // { // FIXME script importing isn't working
  //   testFile: "upstream/encoding/textdecoder-labels.any.js",
  // },
  // { // FIXME need to make a custom test that doesn't run non-UTF8 encodings and doesn't rely on SharedArrayBuffers
  //   testFile: "upstream/encoding/textdecoder-streaming.any.js",
  // },
  // { // FIXME script importing isn't working
  //   testFile: "upstream/encoding/textencoder-constructor-non-utf.any.js",
  // },
  {
    testFile: "upstream/encoding/textencoder-utf16-surrogates.any.js",
  },
  {
    testFile: "upstream/encoding/api-invalid-label.any.js",
  },
];
