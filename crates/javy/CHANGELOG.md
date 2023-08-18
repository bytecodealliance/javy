# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `alloc` module containing implementations of a realloc function and a free function.
- An `export_alloc_fns` crate feature which when enabled, will export `canonical_abi_realloc` and `canonical_abi_free`
  functions from your Wasm module.

## 2.0.0 - 2023-08-17

### Changed

- Update of `quickjs` types to use types in `quickjs-wasm-rs` 2.0.0.
- WASI SDK will be automatically downloaded at build time if `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable is not set.

## 1.0.0 - 2023-05-16

Initial release
