# Dynamically linked modules

An important use for Javy is for when you may want or need to generate much
smaller Wasm modules. Using the `-C dynamic` flag when invoking `javy build` will create
a dynamically linked module which will have a much smaller file size than
a statically linked module. Statically linked modules embed the JS engine inside
the module while dynamically linked modules rely on Wasm imports to provide the
JS engine. Dynamically linked modules have special requirements that statically
linked modules do not and will not execute in WebAssembly runtimes that do not
meet these requirements.

To successfully instantiate and run a dynamically linked Javy module, the
execution environment must provide a `javy_quickjs_provider_v<version>` namespace for
importing that links to the exports provided by the `javy_quickjs_provider.wasm`
module. Dynamically linked modules **cannot** be instantiated in environments
that do not provide this import.

Dynamically linked Javy modules are tied to QuickJS since they use QuickJS's
bytecode representation.


#### Obtaining the provider module

The `javy_quickjs_provider.wasm` module is available as an asset on the Javy
release you are using. 

It can also be obtained by running `javy emit-provider -o
<path>` to write the module into `<path>`.

#### Creating and running a dynamically linked module througy the CLI

Run:

```
$ echo 'console.log("hello world!");' > my_code.js
$ javy build -C dynamic -o my_code.wasm my_code.js
$ javy emit-provider -o provider.wasm
$ wasmtime run --preload javy_quickjs_provider_v3=provider.wasm my_code.wasm
hello world!
```
