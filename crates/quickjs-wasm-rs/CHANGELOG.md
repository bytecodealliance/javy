# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `JSValueRef` can convert to Rust types with `try_into` (previously this was implemented on `CallbackArg`)
- Added `eval_module` method on `JSContextRef` that evaluates JS code in a ECMAScript module scope

### Changed
- Callback functions registered with `context.wrap_callback` now pass `JSValueRef` into the closure instead of `CallbackArg`
- `from_qjs_value` now takes `JSValueRef` by value

### Removed
- `CallbackArg` type

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

[unreleased]: https://github.com/bytecodealliance/javy/compare/quickjs-wasm-rs-1.0.0...HEAD
[1.0.0]: https://github.com/bytecodealliance/javy/tree/quickjs-wasm-rs-1.0.0/crates/quickjs-wasm-rs

