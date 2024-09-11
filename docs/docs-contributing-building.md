# Build requirements

- On Ubuntu, `sudo apt-get install curl pkg-config libssl-dev clang`
- [rustup](https://rustup.rs/)
- Stable Rust, installed via `rustup install stable && rustup default stable`
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`
- cmake, depending on your operating system and architecture, it might not be
  installed by default. On MacOS it can be installed with `homebrew` via `brew
  install cmake`. On Ubuntu, `sudo apt-get install cmake`.
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`

# How to build

In the repository root, run:

```
$ cargo build -p javy-core --target=wasm32-wasi -r
$ cargo build -p javy-cli -r
```

Alternatively if you want to install the `javy` binary globally, at the
repository root, run:
```
$ cargo build -p javy-core --target=wasm32-wasi -r
$ cargo install --path crates/cli
```

If you are going to recompile frequently, you may want to prepend
`CARGO_PROFILE_RELEASE_LTO=off` to cargo build for the CLI to speed up the
build.
