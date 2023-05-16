# Contributing

## Adding additional JS APIs

We will only add JS APIs or accept contributions that add JS APIs that are potentially useful across multiple environments and do not invoke non-[WASI](https://wasi.dev/) hostcalls. If you wish to add or use JS APIs that do not meet these criteria, please use the `quickjs-wasm-rs` crate directly. We may revisit how we support importing and exporting custom functionality from Javy once [the Component Model](https://github.com/WebAssembly/component-model) has stabilized.

## Versioning for library crates

The library crates, `javy`, `javy-apis`, `quickjs-wasm-rs`, and `quickjs-wasm-sys`, use the versioning system described in [Rust for Rustaceans](https://rust-for-rustaceans.com/) in the _Unreleased Versions_ section in the _Project Structure_ chapter. The underlying motivation is that the version in the crate's `Cargo.toml` is important between releases to ensure Cargo does not reuse a stale version if a project relies on a version of the crate that has not yet been published to crates.io and the version required by that project is updated to a version with new additive or breaking changes.

### The system

After publishing a release, immediately update the version number to the next patch version with an `-alpha.1` suffix. The first time an additive change is made, reset the patch version to `0` and increment the minor version and reset the suffix to `-alpha.1`. When making additional additive changes, increment the number in the suffix, for example `-alpha.2`. The first time a breaking change is made, reset the patch version and minor version to `0` and increment the major version and reset the suffix to `-alpha.1`. When making additional breaking changes, increment the number in the suffix, for example `-alpha.2`.

When releasing, remove the suffix and then publish.
