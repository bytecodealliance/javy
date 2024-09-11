### Invoking modules programatically

Javy-generated modules are by design WASI only and follow the [command
pattern](https://github.com/WebAssembly/WASI/blob/snapshot-01/design/application-abi.md#current-unstable-abi).

Any input must be passed via `stdin` and any output will be placed in `stdout`.
This is especially important when invoking Javy modules from a custom embedding. 

In a runtime like Wasmtime, [wasmtime-wasi](
https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/struct.WasiCtx.html#method.set_stdin)
can be used to set the input and retrieve the output.

To embed Javy in a Node.js application see this
[example](./docs-using-nodejs.md).
