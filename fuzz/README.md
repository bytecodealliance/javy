<div align="center">
  <h1><code>javy-fuzz</code></h1>
  <p>
    <strong>Fuzzing infrastructure for Javy</strong>
  </p>
</div>

This crates defines all the fuzzing targets for Javy.

It uses [`libFuzzer`](https://llvm.org/docs/LibFuzzer.html) through [`cargo fuzz`](https://github.com/rust-fuzz/cargo-fuzz)

## Running

```sh
cargo +nightly fuzz run $TARGET
```

## Available Targets

* `json-differential`: Generate valid json and execute `JSON.parse` and
    `JSON.stringify` using Javy's custom, SIMD-based implementation and validate
    it against QuickJS' native implementation.



