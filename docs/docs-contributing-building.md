# Build requirements

- On Ubuntu, `sudo apt-get install curl pkg-config libssl-dev clang`
- [rustup](https://rustup.rs/)
- Stable Rust, installed via `rustup install stable && rustup default stable`
- wasm32-wasip1, can be installed via `rustup target add wasm32-wasip1`
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`

# How to build

In the repository root, run:

```
$ cargo build -p javy-plugin --target=wasm32-wasip1 -r
$ cargo build -p javy-cli -r
```

Alternatively if you want to install the `javy` binary globally, at the
repository root, run:
```
$ cargo build -p javy-plugin --target=wasm32-wasip1 -r
$ cargo install --path crates/cli
```

If you are going to recompile frequently, you may want to prepend
`CARGO_PROFILE_RELEASE_LTO=off` to cargo build for the CLI to speed up the
build.
