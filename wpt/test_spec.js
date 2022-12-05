export default [
  {
    "testFile": "./custom_test.js",
    "ignoredTests": ["This is an ignored test"]
  },
  {
    "testFile": "upstream/encoding/textdecoder-eof.any.js",
    "ignoredTests": ["/stream: true/"]
  },
  {
    "testFile": "upstream/encoding/textencoder-utf16-surrogates.any.js",
  }
]
