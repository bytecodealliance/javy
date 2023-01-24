export default [
  {
    testFile: "./custom_test.js",
    ignoredTests: ["This is an ignored test"],
  },
  {
    testFile: "upstream/encoding/api-basics.any.js",
    ignoredTests: ["Decode sample: utf-16le", "Decode sample: utf-16be", "Decode sample: utf-16"],
  },
  {
    testFile: "upstream/encoding/api-invalid-label.any.js",
  },
  {
    testFile: "upstream/encoding/api-replacement-encodings.any.js",
  },
  {
    testFile: "upstream/encoding/api-surrogates-utf8.any.js",
  },
  // { // FIXME requires `encodeInto` support
  //   testFile: "upstream/encoding/encodeInto.any.js",
  // },
  {
    testFile: "upstream/encoding/replacement-encodings.any.js",
  },
  // { // FIXME need to add streaming support
  //   testFile: "upstream/encoding/textdecoder-arguments.any.js",
  // },
  {
    testFile: "upstream/encoding/textdecoder-byte-order-marks.any.js",
    ignoredTests: ["Byte-order marks: utf-16le", "Byte-order marks: utf-16be"],
  },
  {
    testFile: "upstream/encoding/textdecoder-eof.any.js",
    ignoredTests: ["/stream: true/"],
  },
  {
    testFile: "custom_tests/textdecoder-fatal-streaming.any.js",
    ignoredTests: ["Fatal flag, streaming cases"]
  },
  {
    testFile: "upstream/encoding/textdecoder-fatal.any.js",
    ignoredTests: ["Fatal flag: utf-16le - truncated code unit"],
  },
  {
    testFile: "upstream/encoding/textdecoder-ignorebom.any.js",
    ignoredTests: ["/utf-16/"]
  },
  {
    testFile: "upstream/encoding/textdecoder-labels.any.js",
    ignoredTests: [
      "/IBM866/", "/ISO-8859-/", "/KOI8-/", "/macintosh/", "/windows-/", "/x-mac-cyrillic/",
      "/GBK/", "/gb18030/", "/Big5/", "/EUC-/", "/ISO-2022-JP/", "/Shift_JIS/", "/UTF-16/",
      "/x-user-defined/",
    ],
  },
  // { // FIXME need to make a custom test that doesn't run non-UTF8 encodings and doesn't rely on SharedArrayBuffers
  //   testFile: "upstream/encoding/textdecoder-streaming.any.js",
  // },
  {
    testFile: "upstream/encoding/textencoder-constructor-non-utf.any.js",
    ignoredTests: [
      "/IBM866/", "/ISO-8859-/", "/KOI8-/", "/macintosh/", "/windows-/", "/x-mac-cyrillic/",
      "/GBK/", "/gb18030/", "/Big5/", "/EUC-/", "/ISO-2022-JP/", "/Shift_JIS/", "/UTF-16/",
      "/x-user-defined/",
    ],
  },
  {
    testFile: "upstream/encoding/textencoder-utf16-surrogates.any.js",
  },
];
