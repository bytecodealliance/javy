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
CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir=." cargo hack test --target=wasm32-wasip1 --workspace --exclude=javy-cli --each-feature -- --nocapture
```
