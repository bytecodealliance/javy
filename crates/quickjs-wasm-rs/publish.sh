#!/usr/bin/env bash

set -e

if [[ -z $QUICKJS_WASM_SYS_WASI_SDK_PATH ]]; then
    echo "QUICKJS_WASM_SYS_WASI_SDK_PATH must be set to a path with a downloaded wasi-sdk" 1>&2
    exit 1
fi

cargo publish --target=wasm32-wasi
