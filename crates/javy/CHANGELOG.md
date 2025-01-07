# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.0.0] - 2025-01-08

### Removed

- `Javy.JSON.fromStdin` and `Javy.JSON.toStdout` APIs and `javy_json` method on
  `javy::Config`.

## [3.1.0] - 2024-11-27

### Added

- `gc_threshold`, `memory_limit`, and `max_stack_size` properties for `Config`.

### Fixed

- Addressed memory leak when registering `JSON.parse` and `JSON.stringify`
  functions.

## [3.0.2] - 2024-11-12

### Changed

- Misc dependency updates

## [3.0.1] - 2024-09-18

### Changed

- Updated `simd-json` to version that removes dependency on `lexical-core` with
  a security vulnerability.

### Fixed

- Circular dependency checks for the custom, SIMD-based, `JSON.stringify`
  (https://github.com/bytecodealliance/javy/pull/700)

## [3.0.0] - 2024-06-12

### Changed

- Introduce `rquickjs` to interface with QuickJS instead of `quickjs-wasm-rs`;
  this version no longer includes re-exports from `quickjs-wasm-rs`.
- `javy::serde::de::Deserializer` should now match `JSON.stringify`: non-JSON
  primitives are skipped in Objects and nullified in Arrays. 
- Introduce a faster implemementation for `JSON.parse` and `JSON.stringify`
  based on `simd-json`, `serde-json` and `serde-transcode`. Also introduce the
  `Javy.JSON` temporary helper namespace which contains helpers for working with
  JSON.

## [2.2.0] - 2024-01-31

### Fixed

- Missing documentation for `export_alloc_fns` feature and `alloc` functions.

### Changed

- Updated to 2023-12-09 release of QuickJS.

## [2.1.0] - 2023-09-11

### Added

- `alloc` module containing implementations of a realloc function and a free
  function.
- An `export_alloc_fns` crate feature which when enabled, will export
  `canonical_abi_realloc` and `canonical_abi_free` functions from your Wasm
  module.

### Fixed

- Documentation now builds on docs.rs.

## [2.0.0] - 2023-08-17

### Changed

- Update of `quickjs` types to use types in `quickjs-wasm-rs` 2.0.0.
- WASI SDK will be automatically downloaded at build time if
  `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable is not set.

## [1.0.0] - 2023-05-16

Initial release
