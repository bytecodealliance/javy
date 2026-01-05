# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Bumped rquickjs to 0.11. If you are using a plugin for dynamic linking, you
  are strongly encouraged to change the import namespace.

### Fixed

- Fixed memory leak in Serde deserializer implementation for RQuickJS values.

## [5.0.0] - 2025-11-12

### Removed

- `big_float`, `big_decimal`, and `string_normalize` on `Config`. For BigFloat
  and BigDecimal, there is no replacement. `string_normalize` is enabled
  unconditionally.

### Added

- `weak_ref` and `performance` on `Config`.
- Support for WASI preview 1 plugins.

### Changed

- The QuickJS bytecode format has changed in a breaking way.

## [4.1.0] - 2025-10-06

### Changed

- `javy` dependency updated. Now exposes intrinsic for `String.normalize` that defaults to enabled.

## [4.0.0] - 2025-08-28

### Added

- `javy_plugin` macro to generate a plugin that uses the new plugin API.

### Changed

- The plugin API expected by the Javy CLI has been changed. The README contains
  a section on how to update your plugin to use the new API. Please read the
  extending Javy documentation for more details on the new plugin API.
- `initialize_runtime`, `compile_src`, and `invoke` function signatures have
  changed.

### Removed

- `run_bytecode` and `eval_bytecode` functions have been removed.

## [3.2.0] - 2025-07-24

### Added

- `javy` dependency updated to 4.1.0 which adds `log_stream` and `err_stream`
  methods to `Config`.

## [3.1.0] - 2025-04-17

### Added

- Added `messagepack` feature exposing javy/messagepack feature

## [3.0.0] - 2025-01-08

### Removed

- `javy` dependency updated to 4.0.0 which removes `javy_json` method on
  `javy_plugin_api::Config` and removes support for `Javy.JSON.fromStdin` and
  `Javy.JSON.toStdout`.

## [2.0.0] - 2024-11-27

### Changed

- `initialize_runtime` accepts a `javy_plugin_api::Config` instead of a
  `javy_plugin_api::javy::Config`

### Added

- Can now enable the event loop using the `javy_plugin_api::Config`

## [1.0.0] - 2024-11-12

Initial release
