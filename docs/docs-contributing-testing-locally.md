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

3. Run tests and linting, eg:
```
make fmt tests
```

4. If adding new dependencies, vet the dependencies

```
cargo vet
```

If this fails, follow on-screen instructions to trust any dependencies it suggests trusting. If `cargo vet` still fails after trusting those dependencies, then run:

```
cargo vet regenerate exemptions
```
