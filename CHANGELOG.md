# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [5.0.4-workato.1] - 2025-05-23

### Added

- **Complete Timer API Support**: Implemented all four browser-standard timer functions
  - `setTimeout(callback, delay)` - Creates one-time delayed execution timers
  - `clearTimeout(id)` - Cancels timeout timers
  - `setInterval(callback, delay)` - Creates repeating timers with automatic rescheduling
  - `clearInterval(id)` - Cancels interval timers
- **WASI P1 Compatible Implementation**: Pure Rust timer system using synchronous polling approach
- **Enhanced CLI Configuration**: Updated help text and documentation
  - `-J timers=y` flag enables all timer APIs

---

## Previous Versions

### [5.0.4] - Base Javy Release

- Base Javy CLI functionality
- Existing JavaScript runtime features
- WASI P1 compatibility 