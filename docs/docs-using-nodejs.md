# Embedding in Node.js Application
This example demonstrates how to run Javy in a Node.js (v20+) host application.

## Warning
This example does NOT show how to run a Node.js application in Javy. This is
useful for when you want to run untrusted user generated code in a sandbox. This
code is meant to be an example not production-ready code. 

It's also important to note that the WASI implementation in NodeJS is currently
considered [experimental].

[experimental]: https://nodejs.org/api/wasi.html#webassembly-system-interface-wasi

## Summary
This example shows how to use a dynamically linked Javy compiled Wasm module. We
use std in/out/error to communicate with the embedded javascript see [this blog
post](https://k33g.hashnode.dev/wasi-communication-between-nodejs-and-wasm-modules-another-way-with-stdin-and-stdout)
for details.


### Steps

1. Emit the Javy plugin
```shell
javy emit-plugin -o plugin.wasm
```
2. Compile the `embedded.js` with Javy using dynamic linking:
```shell
javy build -C dynamic -C plugin=plugin.wasm -o embedded.wasm embedded.js
```
3. Run `host.mjs`
```shell
node --no-warnings=ExperimentalWarning host.mjs
```


`embedded.js`
```javascript
// Read input from stdin
const input = readInput();
// Call the function with the input
const result = foo(input);
// Write the result to stdout
writeOutput(result);

// The main function.
function foo(input) {
  if (input && typeof input === "object" && typeof input.n === "number") {
    return { n: input.n + 1 };
  }
  return { n: 0 };
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
  const { finalBuffer } = inputChunks.reduce(
    (context, chunk) => {
      context.finalBuffer.set(chunk, context.bufferOffset);
      context.bufferOffset += chunk.length;
      return context;
    },
    { bufferOffset: 0, finalBuffer: new Uint8Array(totalBytes) },
  );

  const maybeJson = new TextDecoder().decode(finalBuffer);
  try {
    return JSON.parse(maybeJson);
  } catch {
    return;
  }
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


`host.mjs`
```javascript
import { readFile, writeFile, open } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { WASI } from "wasi";

try {
  const [embeddedModule, pluginModule] = await Promise.all([
    compileModule("./embedded.wasm"),
    compileModule("./plugin.wasm"),
  ]);
  const result = await runJavy(pluginModule, embeddedModule, { n: 100 });
  console.log("Success!", JSON.stringify(result, null, 2));
} catch (e) {
  console.log(e);
}

async function compileModule(wasmPath) {
  const bytes = await readFile(new URL(wasmPath, import.meta.url));
  return WebAssembly.compile(bytes);
}

async function runJavy(pluginModule, embeddedModule, input) {
  const uniqueId = crypto.randomUUID();

  // Use stdin/stdout/stderr to communicate with Wasm instance
  // See https://k33g.hashnode.dev/wasi-communication-between-nodejs-and-wasm-modules-another-way-with-stdin-and-stdout
  const workDir = tmpdir();
  const stdinFilePath = join(workDir, `stdin.wasm.${uniqueId}.txt`);
  const stdoutFilePath = join(workDir, `stdout.wasm.${uniqueId}.txt`);
  const stderrFilePath = join(workDir, `stderr.wasm.${uniqueId}.txt`);

  // 👋 send data to the Wasm instance
  await writeFile(stdinFilePath, JSON.stringify(input), { encoding: "utf8" });

  const [stdinFile, stdoutFile, stderrFile] = await Promise.all([
    open(stdinFilePath, "r"),
    open(stdoutFilePath, "a"),
    open(stderrFilePath, "a"),
  ]);

  try {
    const wasi = new WASI({
      version: "preview1",
      args: [],
      env: {},
      stdin: stdinFile.fd,
      stdout: stdoutFile.fd,
      stderr: stderrFile.fd,
      returnOnExit: true,
    });

    const pluginInstance = await WebAssembly.instantiate(
      pluginModule,
      wasi.getImportObject(),
    );
    const instance = await WebAssembly.instantiate(embeddedModule, {
      javy_quickjs_provider_v3: pluginInstance.exports,
    });

    // Javy plugin is a WASI reactor see https://github.com/WebAssembly/WASI/blob/main/legacy/application-abi.md?plain=1
    wasi.initialize(pluginInstance);
    instance.exports._start();

    const [out, err] = await Promise.all([
      readOutput(stdoutFilePath),
      readOutput(stderrFilePath),
    ]);

    if (err) {
      throw new Error(err);
    }

    return out;
  } catch (e) {
    if (e instanceof WebAssembly.RuntimeError) {
      const errorMessage = await readOutput(stderrFilePath);
      if (errorMessage) {
        throw new Error(errorMessage);
      }
    }
    throw e;
  } finally {
    await Promise.all([
      stdinFile.close(),
      stdoutFile.close(),
      stderrFile.close(),
    ]);
  }
}

async function readOutput(filePath) {
  const str = (await readFile(filePath, "utf8")).trim();
  try {
    return JSON.parse(str);
  } catch {
    return str;
  }
}
```
