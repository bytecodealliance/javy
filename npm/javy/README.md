# Javy Lib

This library provides abstractions and convenience methods over Javy's built-in functions.

In `javy/fs`:

`readFileSync(fd)`

Returns a `Uint8Array` representation (a byte array) of the contents of a file descriptor.

`writeFileSync(fd, buffer)`

Writes the contents of a `Uint8Array` to a file descriptor.

`STDIO`

Has `Stdin`, `Stdout`, and `Stderr` properties which correspond to the integer file descriptors for standard input, standard output, and standard error for use with `readFileSync` and `writeFileSync`.

## Usage example

```js
import { readFileSync, writeFileSync, STDIO } from `javy/fs`

const textEncoder = new TextEncoder();

const inputBuffer = readFileSync(STDIO.Stdin);
const inputText = new TextDecoder().decode(inputBuffer);

const stdoutContent = `${inputText} -- out`;
const stderrContent = `${inputText} -- err`;

writeFileSync(STDIO.Stdout, textEncoder.encode(stdoutContent));
writeFileSync(STDIO.Stderr, textEncoder.encode(stderrContent));
```

## Tests

To run the tests, run `npm test`. It requires you to have a release build of `javy` and have `wasmtime` installed.

## Publishing

Run `npm run build` before running `npm publish`.
