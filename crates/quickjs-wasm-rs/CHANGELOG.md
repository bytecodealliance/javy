# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2023-05-02

### Added
- Documentation across the crate.
- Added an enum `JSValue` that represents a JavaScript value that can convert to and from Rust types with `try_into`.

### Changed
- Renamed `Value` to `JSValueRef`.
- Renamed `Context` to `JSContextRef`.
- Callback functions now work with `CallbackArg` instead of `JSValueRef` directly. `CallbackArg` can easily convert to Rust types with `try_into`.
- Relationship of `JSValueRef` and `JSContextRef` is now safer with `JSValueRef` containing a reference to `JSContextRef` instead of a raw pointer to the quickjs `JSContext`.


[unreleased]: https://github.com/Shopify/javy/compare/quickjs-wasm-rs-1.0.0...HEAD
[1.0.0]: https://github.com/Shopify/javy/tree/quickjs-wasm-rs-1.0.0/crates/quickjs-wasm-rs

