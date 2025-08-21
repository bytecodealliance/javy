# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Generator` now has a `producer_version` method so the version in the
  producers custom section can be set.

### Changed

- `Plugin::new` now validates the bytes are a valid plugin and returns a
  result.

## [1.0.0] - 2025-03-10

Initial release
