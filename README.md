# Javy: A *Jav*aScript to WebAssembl*y* toolchain

![Build status](https://github.com/Shopify/javy/actions/workflows/ci.yml/badge.svg?branch=main)

## About this repo

**Introduction**: Run your JavaScript on WebAssembly. Javy takes your JavaScript code, and executes it in a WebAssembly embedded JavaScript runtime.

Javy is currently used for the beta Shopify Scripts platform. We intend on supporting and improving this runtime in that context. Eventually this project should be a good general purpose JavaScript runtime but that is not the current goal.

## Contributing

Javy is a beta project and will be under major development. We welcome feedback, bug reports and bug fixes. We're also happy to discuss feature development but please discuss the features in an issue before contributing. All contributors will be prompted to sign our CLA.

## Build

- [rustup](https://rustup.rs/)
- Stable Rust (`rustup install stable && rustup default stable`)
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

Define your JavaScript like:

```javascript
function foo(input) {
    return { foo: input.n + 1, newBar: input.bar + "!" };
}

Shopify = {
    main: foo,
};
```

Create a WebAssembly binary from your JavaScript by:

```bash
javy index.js -o destination/index.wasm
```

For more information on the commands you can run `javy --help`

You can then execute your WebAssembly binary using a WebAssembly engine:

```bash
$ echo '{ "n": 2, "bar": "baz" }' | wasmtime index.wasm
{"foo":3,"new_bar":"baz!"}%   
```

### Invoking Javy-generated modules programatically

Javy-generated modules are by design WASI only and follow the [command pattern](https://github.com/WebAssembly/WASI/blob/snapshot-01/design/application-abi.md#current-unstable-abi). Any input must be passed via `stdin` and any output will be placed in `stdout`. This is especially important when invoking Javy modules from a custom embedding. 

In a runtime like Wasmtime, [wasmtime-wasi](
https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/struct.WasiCtx.html#method.set_stdin)
can be used to set the input and retrieve the output.

## Using quickjs-wasm-rs to build your own toolchain

The `quickjs-wasm-rs` crate that is part of this project can be used as part of a Rust crate targeting Wasm to customize how that Rust crate interacts with QuickJS. This may be useful when trying to use JavaScript inside a Wasm module and Javy does not fit your needs as `quickjs-wasm-rs` contains serializers that make it easier to send structured data (for example, strings or objects) between host code and Wasm code.

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
