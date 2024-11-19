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
execution environment must provide a collection of imports that match the
exports from a Javy plugin Wasm module (for example,
`canonical_abi_realloc` and `invoke`). The namespace for these imports must
match the import namespace specified by the Javy plugin Wasm module used to
generate the dynamically linked Wasm module (this is, if the plugin's import
namespace was `my_plugin_v1`, then the imports must be made available under the
module name `my_plugin_v1`). This value is available from the `import_namespace`
custom section in the Javy plugin module. You can also statically inspect the
imports of the dynamically linked Wasm module to determine the import
namespace. Dynamically linked modules **cannot** be instantiated in
environments that do not provide the required imports.

Dynamically linked Javy modules are tied to QuickJS since they use QuickJS's
bytecode representation.

#### Obtaining the default plugin module

The `plugin.wasm` module is available as an asset on the Javy release you are
using. 

It can also be obtained by running `javy emit-plugin -o <path>` to write the
module into `<path>`.

#### Creating and running a dynamically linked module through the CLI

Run:

```
$ echo 'console.log("hello world!");' > my_code.js
$ javy emit-plugin -o plugin.wasm
$ javy build -C dynamic -C plugin=plugin.wasm -o my_code.wasm my_code.js
$ wasmtime run --preload javy_quickjs_provider_v3=plugin.wasm my_code.wasm
hello world!
```
