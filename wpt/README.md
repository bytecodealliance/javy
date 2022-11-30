# Web Platform Tests

This will eventually allow us to run select Web Platform Tests against javy.

## Setup

```
$ git submodule init
$ git submodule update
$ npm i
# Replace the test with any other test you want to run.
$ npx rollup -c rollup.config.js upstream/encoding/textdecoder-fatal-single-byte.any.js
$ javy -o module.wasm bundle.js
```
