# Contributing

## Architecture

See our [architecture document](contributing-architecture.md) for more information about how the project is organized.

## Adding additional JS APIs

We will only add JS APIs or accept contributions that add JS APIs that are potentially useful across multiple environments and do not invoke non-[WASI](https://wasi.dev/) hostcalls. If you wish to add or use JS APIs that do not meet these criteria, please use the `quickjs-wasm-rs` crate directly. We may revisit how we support importing and exporting custom functionality from Javy once [the Component Model](https://github.com/WebAssembly/component-model) has stabilized.

## Versioning for library crates

The library crates, `javy`, `javy-apis`, `quickjs-wasm-rs`, and `quickjs-wasm-sys`, use the versioning system described in [Rust for Rustaceans](https://rust-for-rustaceans.com/) in the _Unreleased Versions_ section in the _Project Structure_ chapter. The underlying motivation is that the version in the crate's `Cargo.toml` is important between releases to ensure Cargo does not reuse a stale version if a project relies on a version of the crate that has not yet been published to crates.io and the version required by that project is updated to a version with new additive or breaking changes.

The versions for `javy` and `javy-apis` must always be the same. So a version change in the one crate should result in a version change in the other crate.

### The system

After publishing a release, immediately update the version number to the next patch version with an `-alpha.1` suffix. The first time an additive change is made, reset the patch version to `0` and increment the minor version and reset the suffix to `-alpha.1`. When making additional additive changes, increment the number in the suffix, for example `-alpha.2`. The first time a breaking change is made, reset the patch version and minor version to `0` and increment the major version and reset the suffix to `-alpha.1`. When making additional breaking changes, increment the number in the suffix, for example `-alpha.2`.

When releasing, remove the suffix and then publish.

For example, let's say the last published version of `quickjs-wasm-rs` is `2.0.0`, so the current version in the Cargo.toml file is `2.0.0-alpha.1`. If you add a new public function, you would change the version to `2.1.0-alpha.1`. This is because adding a new public function is considered an additive change. After merging those changes, if you add a new public function, you would change the version to `2.1.0-alpha.2`. This is because adding another new function is an additional additive change. Now if you were to make a function that was public, private, you would change the version to `3.0.0-alpha.1`. This is because removing a public function is considered a breaking change. After merging that change, if you were to then add a new public function, then you would increment the version to `3.0.0-alpha.2` because this is making an additional additive change. It's not necessary to increment the minor version in this case because version `3.0.0` has not been published yet so version `3.0.0` can contain a mixture of additive and breaking changes from the last `2.x.x` version published.

## cargo vet

We use [cargo vet](https://mozilla.github.io/cargo-vet/) to audit dependencies for the project. If you need to change or add dependencies, please try to use a dependency that has been audited by one one of the audits we import or is published by one of the authors we trust (sunfishcode, dtolnay, Amanieu, cuviper). This is preferable to adding new exemptions for the project. Do not add new audits for crates that are not in this project.

## Web platform tests (WPT)

We run a subset of the web platform test suite during continuous integration. We recommend reading our suite's [WPT readme](../wpt/README.md) for tips on how to add tests to the suite and what to do when tests won't pass.
