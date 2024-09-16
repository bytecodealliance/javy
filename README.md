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

**Introduction**: Run your JavaScript on WebAssembly. Javy takes your JavaScript
code, and executes it in a WebAssembly embedded JavaScript runtime. Javy can
create _very_ small Wasm modules in the 1 to 16 KB range with use of dynamic
linking. The default static linking produces modules that are at least 869 KB in
size.

## Installation

Pre-compiled binaries of the Javy CLI can be found on [the releases
page](https://github.com/bytecodealliance/javy/releases).

## Example

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
javy build index.js -o destination/index.wasm
```

For more information on the commands you can run `javy --help`

You can then execute your WebAssembly binary using a WebAssembly engine:

```bash
$ echo '{ "n": 2, "bar": "baz" }' | wasmtime index.wasm
{"foo":3,"newBar":"baz!"}%   
```

## Documentation

Read the documentation [here](./docs/index.md)
