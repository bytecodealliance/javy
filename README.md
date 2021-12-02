# Javy: A _Jav_ aScript to WebAssembl _y_  toolchain

![Build status](https://github.com/Shopify/javy/actions/workflows/ci.yml/badge.svg?branch=main)

[About this repo](#about-this-repo) | [Contributing](#contributing) | [Dependencies](#dependencies) | [Releasing](#releasing) | [Compiling to WebAssembly](#compiling-to-webassembly)

## About this repo

**Introduction**: Run your JavaScript on WebAssembly. Javy takes your JavaScript code, and executes it in a WebAssebmly embedded JavaScript runtime.

Javy is currently used for the beta Shopify Scripts platform. We intend on supporting and improving this runtime in that context. Eventually this project should be a good general purpose JavaScript runtime but that is not the current goal.

## Contributing

Javy is a beta project and will be under major development. We welcome feedback, bug reports and bug fixes. We're also happy to discuss feature development but please discuss the features in an issue before contributing. All contributors will be prompted to sign our CLA.

## Dependencies

**Building**
- Rust v1.53.0
- [rustup](https://rustup.rs/)
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`

**Development**

- wasmtime-cli, can be installed via `cargo install wasmtime-cli` (required for
  `cargo-wasi`)
- cargo-wasi, can be installed via `cargo install cargo-wasi`

## Building

After all the dependencies are installed, run `make`. You
should now have access to the executable in `target/release/javy`

Alternatively you can run `make && cargo install --path crates/cli`.
After running the previous command you'll have a global installation of the
executable.

## Releasing

- Create a [new release](https://github.com/Shopify/javy/releases/new) in GitHub. A GitHub Action will be automatically triggered to compile Javy and upload the binaries and hashes to the new Release.

**Updating the Shopify-CLI**
- In the [Shopify CLI](https://github.com/Shopify/shopify-cli), update the version of Javy we want to download [here](https://github.com/Shopify/shopify-cli/blob/bb3f891f3a035c439555621ecaf2cbfa80ac1789/ext/javy/version).
- Copy the generated sha256 hashes (`*.gz.sha256` files) for each distribution into the [hashes folder](https://github.com/Shopify/shopify-cli/tree/bb3f891f3a035c439555621ecaf2cbfa80ac1789/ext/javy/hashes) in the CLI. The hashes can be found in the GitHub Release once the publish GitHub Action completes.
- If required, update the [Javy wrapper](https://github.com/Shopify/shopify-cli/blob/bb3f891f3a035c439555621ecaf2cbfa80ac1789/ext/javy/javy.rb) to support newly released functionality. This is only necessary if a new command was added or the command interface changes.

## Compiling to WebAssembly

You can create a WebAssembly binary from JavaScript by:

```bash
javy index.js -o destination/index.wasm
```

For more information on the commands you can run `javy --help`
