# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Rewrite the APIs on top of Javy v3.0.0, which drops support for
  `quickjs-wasm-rs` in favor of `rquickjs`

## [2.2.0] - 2024-01-31

### Changed

- Updated to 2023-12-09 release of QuickJS.

## [2.1.0] - 2023-09-11

### Fixed

- Documentation builds on docs.rs.

## [2.0.0] - 2023-08-17

### Added

- `random` feature to override `Math.random` implementation with one that sets the random seed on first use of `Math.random`.

### Changed

- `javy` dependency is now at version 2.0.0.
- WASI SDK will be automatically downloaded at build time if `QUICKJS_WASM_SYS_WASI_SDK_PATH` environment variable is not set.

## [1.0.0] - 2023-05-17

Initial release
