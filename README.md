# Javy: A _Jav_ aScript to WebAssembl _y_  toolchain

![Build status](https://github.com/Shopify/javy/actions/workflows/ci.yml/badge.svg?branch=main)


[About this repo](#about-this-repo) | [How to use this repo](#how-to-use-this-repo)

## About this repo

**Introduction**: Run your JavaScript on WebAssembly. Javy takes your
JavaScript code, and executes it in a WebAssebmly embedded JavaScript runtime.

|                |                                                                   |
|----------------|------------------------------------------------------------------:|
| Current status |                                                           Ongoing |
| Owner          |                                                  @Shopify/scripts |
| Help           | [#scripts](https://shopify.slack.com/archives/CE5ENTT7W) on Slack |


## How to use this repo

#### Requirements

##### Build
- Rust v1.53.0
- [rustup](https://rustup.rs/)
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`

##### Development
- wasmtime-cli, can be installed via `cargo install wasmtime-cli` (required for
  `cargo-wasi`)
- cargo-wasi, can be installed via `cargo install cargo-wasi`

#### Building

After all the dependencies are installed, run `make`. You
should now have access to the executable in `target/release/javy`

Alternatively you can run `make && cargo install --path crates/cli`.
After running the previous command you'll have a global installation of the
executable.

#### Compiling to WebAssembly

You can create a WebAssembly binary from JavaScript by:


```bash
javy index.js -o destination/index.wasm
```

For more information on the commands you can run `javy --help`
