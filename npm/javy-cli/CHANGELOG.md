# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.2.0] - 2023-08-17

### Removed

- Building Javy from source code by using the `FORCE_FROM_SOURCE` environment variable is no longer supported.

## [0.1.8] - 2023-07-28

### Fixed

- HTTP response status codes other than 200 when downloading Javy binary now throws an error.
