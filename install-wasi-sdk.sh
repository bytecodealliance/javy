#!/usr/bin/env bash

set -euo pipefail

cd crates/quickjs-wasm-sys
./install-wasi-sdk.sh
