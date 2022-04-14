# Javy: A *Jav*aScript to WebAssembl*y* toolchain

![Build status](https://github.com/Shopify/javy/actions/workflows/ci.yml/badge.svg?branch=main)

## About this repo

**Introduction**: Run your JavaScript on WebAssembly. Javy takes your JavaScript code, and executes it in a WebAssembly embedded JavaScript runtime.

Javy is currently used for the beta Shopify Scripts platform. We intend on supporting and improving this runtime in that context. Eventually this project should be a good general purpose JavaScript runtime but that is not the current goal.

## Contributing

Javy is a beta project and will be under major development. We welcome feedback, bug reports and bug fixes. We're also happy to discuss feature development but please discuss the features in an issue before contributing. All contributors will be prompted to sign our CLA.

## Build

- Rust v1.53.0
- [rustup](https://rustup.rs/)
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`
- cmake, depending on your operating system and architecture, it might not be
  installed by default. On Mac it can be installed with `homebrew` via `brew
  install cmake`
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`
- Install the `wasi-sdk` by running `make download-wasi-sdk`

## Development

- wasmtime-cli, can be installed via `cargo install wasmtime-cli` (required for
  `cargo-wasi`)
- cargo-wasi, can be installed via `cargo install cargo-wasi`

## Building

After all the dependencies are installed, run `make`. You
should now have access to the executable in `target/release/javy`

Alternatively you can run `make && cargo install --path crates/cli`.
After running the previous command you'll have a global installation of the
executable.

## Compiling to WebAssembly

You can create a WebAssembly binary from JavaScript by:

```bash
javy index.js -o destination/index.wasm
```

For more information on the commands you can run `javy --help`

## Releasing

1. Create a tag for the new version like `v0.2.0`
```
git tag v0.2.0
git push origin --tags
```
2. Create a new release from the new tag in github [here](https://github.com/Shopify/javy/releases/new).
3. A GitHub Action will trigger for `publish.yml` when a release is published ([i.e. it doesn't run on drafts](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#:~:text=created%2C%20edited%2C%20or%20deleted%20activity%20types%20for%20draft%20releases)), creating the artifacts for downloading. However this does not currently support `arm-macos`, ie. M1 Macs.
4. Manually build this on a m1 mac

```
gzip -k -f target/release/javy && mv target/release/javy.gz javy-arm-macos-v0.2.0.gz

```
5. Manually create the shasum file
```
shasum -a 256 javy-arm-macos-v0.2.0.gz | awk '{ print $1 }' > javy-arm-macos-v0.2.0.gz.sha256
```
6.  Attach both files to the new release page
