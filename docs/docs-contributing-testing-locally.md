## Testing locally

1. Clone submodules

```
git submodules init
git submodules update
```

2. Install cargo hack

```
cargo +stable install cargo-hack --locked
```

3. Run tests, eg:

```
cargo +stable install cargo-hack --locked
```

```
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=." cargo hack wasi test --workspace --exclude=javy-cli --exclude=javy-config --each-feature -- --nocapture
```
