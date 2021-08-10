Javy CLI embeds a few pre-build dependencies used to optimize the final WASM module.

To update them, simply download an extract the different packages for their respective platform and copy the binary `in vendor/{platform}/{binary}`.

wasm-strip: [wabt](https://github.com/WebAssembly/wabt/releases)
wasm-opt: [binaryen](https://github.com/WebAssembly/binaryen/releases)
