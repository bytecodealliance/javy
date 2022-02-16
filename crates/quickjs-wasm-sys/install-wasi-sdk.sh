#!/usr/bin/env bash

set -euo pipefail

if [[ "$(basename $(pwd))" != "quickjs-wasm-sys" ]]; then
    echo "Run this inside in the quickjs-wasm-sys crate" 1>&2
    exit 1
fi

if [[ ! -d "wasi-sdk" ]]; then
    TMPGZ=$(mktemp)
    VERSION_MAJOR="12"
    VERSION_MINOR="0"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${VERSION_MAJOR}/wasi-sdk-${VERSION_MAJOR}.${VERSION_MINOR}-macos.tar.gz -O $TMPGZ
    else
        wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${VERSION_MAJOR}/wasi-sdk-${VERSION_MAJOR}.${VERSION_MINOR}-linux.tar.gz -O $TMPGZ
    fi
    mkdir wasi-sdk
    tar xf $TMPGZ -C wasi-sdk --strip-components=1
fi
