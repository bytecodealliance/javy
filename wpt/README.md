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

Test suites can be added in `test_spec.js`. Individual tests can be ignored by including their name in the test's ignore list.

## Tips for getting tests to pass

- Adding tests to the ignored list is acceptable if there is no intent to support the feature the test is testing.
- We highly recommend running the WPT suite with the `experimental_event_loop` Cargo feature enabled on the `javy-core` crate so tests relying on the event loop are able to pass.
- Strongly consider adding tests in Rust for APIs you're adding to get faster feedback on failures the WPT suite catches while working on a fix.

### If you need to change upstream tests

- You may need to copy the test into the `custom_tests` directory and make small changes, then have the `test_spec.js` file run the copied test file instead of the upstream one.
  - An example of this is commenting out small parts of test cases that are testing functionality that is intentionally not supported (for example, UTF-16 support for `TextDecoder`).

[wpt]: https://wpt.fyi
[rollup]: https://rollupjs.org
