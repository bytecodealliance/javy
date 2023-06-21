#!/usr/bin/env bash

set -e

cargo publish --target=wasm32-wasi
