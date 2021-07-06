# Javy: A _Jav_ aScript to WebAssembl _y_  toolchain

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

- Rust v1.52.0

- `wasm-strip`: https://github.com/WebAssembly/wabt, which can be installed
  via `brew install wabt`

- `wasm-opt`: https://github.com/WebAssembly/binaryen
  - Download a release for your platform from https://github.com/WebAssembly/binaryen/releases/
  - Put the binaries in `bin` under `/usr/local/bin`
  - Put the binaries in `include` under `/usr/local/include`
  - Put the binaries in `lib` under `/usr/local/include`


#### Building

After all the dependencies are installed, run `make profile=release`. You
should now have access to the executable in `target/release/javy`

#### Compiling to WebAssembly

You can create a WebAssembly binary from JavaScript by:


```bash
javy index.js -o destination/index.wasm
```

For more information on the commands you can run `javy --help`
