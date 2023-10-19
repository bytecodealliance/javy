# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed
- Make `JSContextRef::wrap_rust_value` private. Similar to
  `context::get_rust_value` this function is simply an internal detail.

## [2.0.1] - 2023-09-11

### Fixed

- Documentation now builds on docs.rs

## [2.0.0] - 2023-08-17

### Added

- `JSValueRef` can convert to Rust types with `try_into` (previously this was implemented on `CallbackArg`).
- Added `eval_module` method on `JSContextRef` that evaluates JS code in a ECMAScript module scope.

### Changed

- Callback functions registered with `context.wrap_callback` now pass `JSValueRef` into the closure instead of `CallbackArg`.
- `from_qjs_value` now takes `JSValueRef` by value.
- Updated to `quickjs-wasm-sys` version `1.1.0` which will automatically download a WASI SDK if the `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable is not set.

### Removed

- `CallbackArg` type.

## [1.0.0] - 2023-05-05

### Added

- Documentation across the crate.
- Added an enum `JSValue` that represents a JavaScript value that can convert to and from Rust types with `try_into`.

### Changed

- Renamed `Value` to `JSValueRef`.
- Renamed `Context` to `JSContextRef`.
- Callback functions now work with `CallbackArg` instead of `JSValueRef` directly. `CallbackArg` can easily convert to Rust types with `try_into`.
- Relationship of `JSValueRef` and `JSContextRef` is now safer with `JSValueRef` containing a reference to `JSContextRef` instead of a raw pointer to the quickjs `JSContext`.

### Removed

- `json` and `messagepack` features have been moved to the `javy` crate
