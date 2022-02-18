#!/usr/bin/env bash

set -euo pipefail

if [[ "$(basename $(pwd))" != "javy" ]]; then
    echo "Run this inside in the root of the javy repo" 1>&2
    exit 1
fi

PATH_TO_SDK="crates/quickjs-wasm-sys/wasi-sdk"
if [[ ! -d $PATH_TO_SDK ]]; then
    TMPGZ=$(mktemp)
    VERSION_MAJOR="12"
    VERSION_MINOR="0"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${VERSION_MAJOR}/wasi-sdk-${VERSION_MAJOR}.${VERSION_MINOR}-macos.tar.gz -O $TMPGZ
    else
        wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${VERSION_MAJOR}/wasi-sdk-${VERSION_MAJOR}.${VERSION_MINOR}-linux.tar.gz -O $TMPGZ
    fi
    mkdir $PATH_TO_SDK
    tar xf $TMPGZ -C $PATH_TO_SDK --strip-components=1
fi
