<div align="center">
  <h1><code>Javy</code></h1>

  <p>
    <strong>A <i>Jav</i>aScript to Webassembl<i>y</i> toolchain</strong>
  </p>

  <strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

  <p>
    <a href="https://github.com/bytecodealliance/javy/actions/workflows/ci.yml"><img alt="Build status" src="https://github.com/bytecodealliance/javy/actions/workflows/ci.yml/badge.svg?branch=main" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/370816-javy"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
  </p>
</div>

## About this repo

**Introduction**: Run your JavaScript on WebAssembly. Javy takes your JavaScript code, and executes it in a WebAssembly embedded JavaScript runtime. Javy can create _very_ small Wasm modules in the 1 to 16 KB range with use of dynamic linking. The default static linking produces modules that are at least 869 KB in size.

## Runtime requirements

When running the official Javy binary on Linux, `glibc` 2.31 or greater must be available. You may need to update the version of your operating system if you are using an older version of `glibc`.

## Contributing

We welcome feedback, bug reports and bug fixes. We're also happy to discuss feature development but please discuss the features in an issue before contributing.

Read our [contribution documentation](docs/contributing.md) for additional information on contributing to Javy.

## Requirements to build

- On Ubuntu, `sudo apt-get install curl pkg-config libssl-dev clang`
- [rustup](https://rustup.rs/)
- Stable Rust, installed via `rustup install stable && rustup default stable`
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`
- cmake, depending on your operating system and architecture, it might not be
  installed by default. On MacOS it can be installed with `homebrew` via `brew
  install cmake`. On Ubuntu, `sudo apt-get install cmake`.
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`

## Development requirements

- wasmtime-cli, can be installed via `cargo install wasmtime-cli` (required for
  `cargo-wasi`)
- cargo-wasi, can be installed via `cargo install cargo-wasi`
- cargo-hack, can be installed via `cargo +stable install cargo-hack --locked`

## How to build

Inside the Javy repository, run:
```
$ cargo build -p javy-core --target=wasm32-wasi -r
$ cargo build -p javy-cli -r
```

Alternatively if you want to install the Javy CLI globally, inside the Javy repository run:
```
$ cargo build -p javy-core --target=wasm32-wasi -r
$ cargo install --path crates/cli
```

## Using Javy

Pre-compiled binaries of the Javy CLI can be found on [the releases page](https://github.com/bytecodealliance/javy/releases).

Javy supports ECMA2020 JavaScript. Javy does _not_ provide support for NodeJS or CommonJS APIs.

### Compiling to WebAssembly

Define your JavaScript like:

```javascript
// Read input from stdin
const input = readInput();
// Call the function with the input
const result = foo(input);
// Write the result to stdout
writeOutput(result);

// The main function.
function foo(input) {
    return { foo: input.n + 1, newBar: input.bar + "!" };
}

// Read input from stdin
function readInput() {
    const chunkSize = 1024;
    const inputChunks = [];
    let totalBytes = 0;

    // Read all the available bytes
    while (1) {
        const buffer = new Uint8Array(chunkSize);
        // Stdin file descriptor
        const fd = 0;
        const bytesRead = Javy.IO.readSync(fd, buffer);

        totalBytes += bytesRead;
        if (bytesRead === 0) {
            break;
        }
        inputChunks.push(buffer.subarray(0, bytesRead));
    }

    // Assemble input into a single Uint8Array
    const { finalBuffer } = inputChunks.reduce((context, chunk) => {
        context.finalBuffer.set(chunk, context.bufferOffset);
        context.bufferOffset += chunk.length;
        return context;
    }, { bufferOffset: 0, finalBuffer: new Uint8Array(totalBytes) });

    return JSON.parse(new TextDecoder().decode(finalBuffer));
}

// Write output to stdout
function writeOutput(output) {
    const encodedOutput = new TextEncoder().encode(JSON.stringify(output));
    const buffer = new Uint8Array(encodedOutput);
    // Stdout file descriptor
    const fd = 1;
    Javy.IO.writeSync(fd, buffer);
}
```

Create a WebAssembly binary from your JavaScript by:

```bash
javy compile index.js -o destination/index.wasm
```

For more information on the commands you can run `javy --help`

You can then execute your WebAssembly binary using a WebAssembly engine:

```bash
$ echo '{ "n": 2, "bar": "baz" }' | wasmtime index.wasm
{"foo":3,"newBar":"baz!"}%   
```

### Exporting functions

To export exported JavaScript functions, you can pass a WIT file and WIT world when running `javy compile`. Only ESM exports are supported (that is, Node.js/CommonJS exports are _not_ supported). For each exported JavaScript function, Javy will add an additional function export to the WebAssembly module. Exported functions with arguments and generators are not supported. Return values will also be dropped and not returned. The Wasm module generated is a core Wasm module, **not** a Wasm component.

An example looks like:

`index.js`:
```javascript
export function foo() {
  console.log("Hello from foo!");
}

console.log("Hello world!");
```

`index.wit`:
```
package local:main

world index-world {
  export foo: func() 
}
```

In the terminal:
```bash
$ javy compile index.js --wit index.wit -n index-world -o index.wasm
$ wasmtime run --invoke foo index.wasm
Hello world!
Hello from foo!
```

The WIT package name and WIT world name do not matter as long as they are present and syntactically correct WIT (that is, it needs to be two names separated by a `:`). The name of the WIT world (that is, the value after `world` and before `{`) must be passed as the `-n` argument. The `-n` argument identifies the WIT world in the WIT file for the Wasm module generated by `javy compile`.

### Invoking Javy-generated modules programatically

Javy-generated modules are by design WASI only and follow the [command pattern](https://github.com/WebAssembly/WASI/blob/snapshot-01/design/application-abi.md#current-unstable-abi). Any input must be passed via `stdin` and any output will be placed in `stdout`. This is especially important when invoking Javy modules from a custom embedding. 

In a runtime like Wasmtime, [wasmtime-wasi](
https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/struct.WasiCtx.html#method.set_stdin)
can be used to set the input and retrieve the output.

### Creating and using dynamically linked modules

An important use for Javy is for when you may want or need to generate much smaller Wasm modules. Using the `-d` flag when invoking Javy will create a dynamically linked module which will have a much smaller file size than a statically linked module. Statically linked modules embed the JS engine inside the module while dynamically linked modules rely on Wasm imports to provide the JS engine. Dynamically linked modules have special requirements that statically linked modules do not and will not execute in WebAssembly runtimes that do not meet these requirements.

To successfully instantiate and run a dynamically linked Javy module, the execution environment must provide a `javy_quickjs_provider_v1` namespace for importing that links to the exports provided by the `javy_quickjs_provider.wasm` module. Dynamically linked modules **cannot** be instantiated in environments that do not provide this import.

Dynamically linked Javy modules are tied to QuickJS since they use QuickJS's bytecode representation.

#### Obtaining the QuickJS provider module

The `javy_quickjs_provider.wasm` module is available as an asset on the Javy release you are using. It can also be obtained by running `javy emit-provider -o <path>` to write the module into `<path>`.

#### Creating and running a dynamically linked module on the CLI

```
$ echo 'console.log("hello world!");' > my_code.js
$ javy compile -d -o my_code.wasm my_code.js
$ javy emit-provider -o provider.wasm
$ wasmtime run --preload javy_quickjs_provider_v1=provider.wasm my_code.wasm
hello world!
```

## Using quickjs-wasm-rs to build your own toolchain

The `quickjs-wasm-rs` crate that is part of this project can be used as part of a Rust crate targeting Wasm to customize how that Rust crate interacts with QuickJS. This may be useful when trying to use JavaScript inside a Wasm module and Javy does not fit your needs as `quickjs-wasm-rs` contains serializers that make it easier to send structured data (for example, strings or objects) between host code and Wasm code.

## Releasing

1. Update the root `Cargo.toml` with the new version
2. Create a tag for the new version like `v0.2.0`
```
git tag v0.2.0
git push origin --tags
```
3. Create a new release from the new tag in github [here](https://github.com/bytecodealliance/javy/releases/new).
4. A GitHub Action will trigger for `publish.yml` when a release is published ([i.e. it doesn't run on drafts](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#:~:text=created%2C%20edited%2C%20or%20deleted%20activity%20types%20for%20draft%20releases)), creating the artifacts for downloading.
