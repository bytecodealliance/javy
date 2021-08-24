# Javy: A _Jav_ aScript to WebAssembl _y_  toolchain

[![Build status](https://badge.buildkite.com/7f78e611f58950fa1d3f26b3486c941bc9a104f593ccf57fa8.svg)](https://buildkite.com/shopify/javy)

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

- Rust v1.53.0
- [rustup](https://rustup.rs/)
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`


#### Building

Set `JAVY_SKIP_ENGINE_OPTIMIZATIONS=1` to disable engine optimizations
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
