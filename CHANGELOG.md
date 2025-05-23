# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [5.0.4-workato.3] - 2025-05-23

### Added

- **Console.warn API**: Implemented missing console.warn functionality
  - `console.warn(...)` - Outputs warning messages to stderr
  - Full WHATWG Console Standard compliance
  - Same argument patterns as console.log and console.error
- **WASI-P1 stderr Support**: Confirmed working stderr integration
  - `console.warn` → stderr (always)
  - `console.error` → stderr (always)  
  - `console.log` → stdout (normal) or stderr (redirected)
- **CLI Redirect Control**: New `-J redirect-stdout-to-stderr` option for output routing
  - `-J redirect-stdout-to-stderr=y` - All console output goes to stderr
  - `-J redirect-stdout-to-stderr=n` - Normal mode (console.log to stdout)
  - `-J redirect-stdout-to-stderr` - Shorthand for enabling redirect
  - Perfect for containerized environments and log processing pipelines

### Changed

- Enhanced console module architecture to support three separate streams
- Updated runtime configuration to handle redirect functionality
- Improved documentation with console.warn usage examples

### Technical Details

- **Stream Routing**: Proper separation of stdout and stderr streams
- **Backward Compatibility**: Zero breaking changes to existing console.log/error
- **Comprehensive Testing**: Full test coverage for both normal and redirect modes
- **Web Standards**: Follows browser console behavior exactly

## [5.0.4-workato.2] - 2025-05-23

### Added

- **Base64 Encoding/Decoding APIs**: Implemented browser-standard base64 functions
  - `btoa(string)` - Encodes binary string to base64 with Latin1 validation
  - `atob(base64String)` - Decodes base64 to binary string with whitespace tolerance
- **Always Available**: Base64 APIs are enabled by default (no configuration flags required)
- **Browser-Standard Behavior**: Full HTML5 specification compliance
  - Proper Latin1 character range validation (0-255)
  - Automatic whitespace filtering in `atob()`
  - Correct error handling for invalid inputs
- **Pure Rust Implementation**: Zero external dependencies with comprehensive test coverage

### Changed

- Base64 APIs are now core functionality like `console.log` (no `-J` flag needed)
- Enhanced developer experience with immediately available encoding/decoding

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