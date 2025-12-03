# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- The `generate` method on `Generator` is now async.

## [3.0.0] - 2025-11-12

### Changed

- `cabi_realloc` will not be removed while generating a statically-linked Wasm module from a JS file.

### Removed

- WIT files without semicolons are no longer supported.

## [2.0.0] - 2025-08-28

### Added

- `Generator` now has a `producer_version` method so the version in the
  producers custom section can be set.

### Changed

- The API plugins are required to conform to has been updated. Please consult
  the extending Javy documentation for the new API.
- `Plugin::new` now validates the bytes are a valid plugin and returns a
  result.
- The `source_compression` method of `Generator` has been replaced with a
  `source_embedding` method which takes `SourceEmbedding` argument, specifying
  whether the source custom section should be omitted, uncompressed, or compressed.

## [1.0.0] - 2025-03-10

Initial release
