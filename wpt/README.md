# Web Platform Tests

This is a bit of a hacky harness to run [Web Platform Tests][wpt] against Javy.

## Setup

Web Platform Tests are included as a submodule.

```
$ git submodule init
$ git submodule update
$ npm i
```

## Testing

This command uses [rollup] with a custom plugin to bundle all selected tests into one bundle. It then uses the local build of `javy` to turn that bundle into a WebAssembly module, and finally runs that WebAssembly module using `wasmtime`.

**Make sure you have `wasmtime` installed and built javy locally.**

If wasmtime generates no output, all tests have been passed. Otherwise, it's a failure.

```
$ npm test
```

## Test selection

Tests can be included in `test_spec.json`. Individual tests can be ignored by including their name in the test's ignore list.

[wpt]: https://wpt.fyi
[rollup]: https://rollupjs.org
