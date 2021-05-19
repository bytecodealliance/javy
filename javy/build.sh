#!/bin/bash
export CC_wasm32_wasi="/opt/wasi-sdk/bin/clang --sysroot=/opt/wasi-sdk/share/wasi-sysroot"
export AR_wasm32_wasi="/opt/wasi-sdk/bin/ar"
export CFLAGS="--sysroot=/opt/wasi-sdk/share/wasi-sysroot"

cargo build --target=wasm32-wasi --release --verbose

if [  -f javy.wat ]; then
  rm javy.wat
fi

if [ -f javy.wasm ]; then
  rm javy.wasm
fi

if [ -f target/wasm32-wasi/release/javy.wasm ]; then
  cp target/wasm32-wasi/release/javy.wasm .
  wasm2wat javy.wasm -o javy.wat
fi

