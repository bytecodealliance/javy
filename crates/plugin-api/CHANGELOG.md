# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.0.0] - 2025r-01-08

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
