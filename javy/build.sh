#!/bin/bash
export CC_wasm32_wasi="/opt/wasi-sdk/bin/clang --sysroot=/opt/wasi-sdk/share/wasi-sysroot"
export AR_wasm32_wasi="/opt/wasi-sdk/bin/ar"
export CFLAGS="--sysroot=/opt/wasi-sdk/share/wasi-sysroot"


if [ -f javy/benches/javy.control.wasm ]; then
  rm javy/benches/javy.control.wasm
fi

if [ -f javy/benches/javy.wizer.wasm ]; then
  rm javy/benches/javy.wizer.wasm
fi

cargo build --target=wasm32-wasi --release

if [ -f target/wasm32-wasi/release/javy.wasm ]; then
  cp target/wasm32-wasi/release/javy.wasm javy/benches/javy.control.wasm
fi

cargo build --target=wasm32-wasi --release --features wizer

if [ -f target/wasm32-wasi/release/javy.wasm ]; then
  wizer --allow-wasi target/wasm32-wasi/release/javy.wasm -o javy/benches/javy.wizer.wasm
fi
