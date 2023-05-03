# Javy: A *Jav*aScript to WebAssembl*y* toolchain

![Build status](https://github.com/Shopify/javy/actions/workflows/ci.yml/badge.svg?branch=main)

## About this repo

**Introduction**: Run your JavaScript on WebAssembly. Javy takes your JavaScript code, and executes it in a WebAssembly embedded JavaScript runtime. Javy can create _very_ small Wasm modules in the 1 to 16 KB range with use of dynamic linking. The default static linking produces modules that are at least 869 KB in size.

## Runtime requirements

When running the official Javy binary on Linux, `glibc` 2.31 or greater must be available. You may need to update the version of your operating system if you are using an older version of `glibc`.

## Contributing

We welcome feedback, bug reports and bug fixes. We're also happy to discuss feature development but please discuss the features in an issue before contributing.

### Adding additional JS APIs

We will only add JS APIs or accept contributions that add JS APIs that are potentially useful across multiple environments and do not invoke non-[WASI](https://wasi.dev/) hostcalls. If you wish to add or use JS APIs that do not meet these criteria, please use the `quickjs-wasm-rs` crate directly. We may revisit how we support importing and exporting custom functionality from Javy once [the Component Model](https://github.com/WebAssembly/component-model) has stabilized.

## Build

- [rustup](https://rustup.rs/)
- Stable Rust (`rustup install stable && rustup default stable`)
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`
- cmake, depending on your operating system and architecture, it might not be
  installed by default. On Mac it can be installed with `homebrew` via `brew
  install cmake`
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`
- Install the `wasi-sdk` by running `make download-wasi-sdk` in the top-level directory in this repository

## Development

- wasmtime-cli, can be installed via `cargo install wasmtime-cli` (required for
  `cargo-wasi`)
- cargo-wasi, can be installed via `cargo install cargo-wasi`
- wizer, can be installed via `cargo install wizer --all-features`

## Building

After all the dependencies are installed, run `make` in the top-level directory of this repository. You
should now have access to the executable in `target/release/javy`.

Alternatively you can run `make && cargo install --path crates/cli`.
After running the previous command you'll have a global installation of the
executable.

## Compiling to WebAssembly

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

### Invoking Javy-generated modules programatically

Javy-generated modules are by design WASI only and follow the [command pattern](https://github.com/WebAssembly/WASI/blob/snapshot-01/design/application-abi.md#current-unstable-abi). Any input must be passed via `stdin` and any output will be placed in `stdout`. This is especially important when invoking Javy modules from a custom embedding. 

In a runtime like Wasmtime, [wasmtime-wasi](
https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/struct.WasiCtx.html#method.set_stdin)
can be used to set the input and retrieve the output.

## Creating and using dynamically linked modules

An important use for Javy is for when you may want or need to generate much smaller Wasm modules. Using the `-d` flag when invoking Javy will create a dynamically linked module which will have a much smaller file size than a statically linked module. Statically linked modules embed the JS engine inside the module while dynamically linked modules rely on Wasm imports to provide the JS engine. Dynamically linked modules have special requirements that statically linked modules do not and will not execute in WebAssembly runtimes that do not meet these requirements.

To successfully instantiate and run a dynamically linked Javy module, the execution environment must provide a `javy_quickjs_provider_v1` namespace for importing that links to the exports provided by the `javy_quickjs_provider.wasm` module. Dynamically linked modules **cannot** be instantiated in environments that do not provide this import.

Dynamically linked Javy modules are tied to QuickJS since they use QuickJS's bytecode representation.

### Obtaining the QuickJS provider module

The `javy_quickjs_provider.wasm` module is available as an asset on the Javy release you are using. It can also be obtained by running `javy emit-provider -o <path>` to write the module into `<path>`.

### Creating and running a dynamically linked module on the CLI

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

1. Create a tag for the new version like `v0.2.0`
```
git tag v0.2.0
git push origin --tags
```
2. Create a new release from the new tag in github [here](https://github.com/Shopify/javy/releases/new).
3. A GitHub Action will trigger for `publish.yml` when a release is published ([i.e. it doesn't run on drafts](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#:~:text=created%2C%20edited%2C%20or%20deleted%20activity%20types%20for%20draft%20releases)), creating the artifacts for downloading.
