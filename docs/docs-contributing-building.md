# Build requirements

- On Ubuntu, `sudo apt-get install curl pkg-config libssl-dev clang`
- [rustup](https://rustup.rs/)
- Stable Rust, installed via `rustup install stable && rustup default stable`
- wasm32-wasip2, can be installed via `rustup target add wasm32-wasip2`
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`

# How to build

In the repository root, run:

```
$ cargo build -p javy-plugin --target=wasm32-wasip2 -r
$ cargo build -p javy-cli
```

Alternatively if you want to install the `javy` binary globally, at the
repository root, run:
```
$ cargo build -p javy-plugin --target=wasm32-wasip2 -r
$ cargo install --path crates/cli
```
