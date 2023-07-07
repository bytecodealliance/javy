# Javy npm package

This is the npm package for Javy. The package contains a small Node script
that downloads the appropriate Javy binary on demand and invokes it with the
parameters given. 

## Usage

```
# Install javy globally
$ npm install -g javy-cli

# Directly invoke it via npm
$ npx javy-cli@latest
```

## Updating javy

The npm package will automatically download the newest version of Javy if a
newer version is available.

## Using a specific version of javy

To use a specific version of Javy, set the environment variable
`FORCE_RELEASE` to the version you would like to use.

```
FORCE_RELEASE=v1.1.0 npx javy-cli@latest
```

## Building from source

If there are no binaries available for your platform or the available binaries
don't work for you for some reason, the npm package can also build Javy from 
source.

```
FORCE_FROM_SOURCE=1 npx javy-cli@latest
```

Please note that for this to work you must have all prerequisites of Javy
(listed in the [README]) installed. (That is CMake, Rust, Rust for wasm32-wasi
target, cargo wasi, wasmtime-cli and Rosetta on Mac M1).

[README]: https://github.com/bytecodealliance/javy/blob/main/README.md


