## Testing locally

1. Clone submodules

```
git submodule init
git submodule update
```

2. Install cargo hack

```
cargo +stable install cargo-hack --locked
```

3. Run tests and linting, eg:
```
make fmt tests
```
