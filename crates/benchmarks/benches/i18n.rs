mod runner;

use crate::runner::Runner;
use criterion::{criterion_group, criterion_main, Criterion};
// One  of the substantial differences between i18n-next and Lisan
// is the amount of code needed to support one vs the other.
//
// With Lisan, the bundled JavaScript is only 3KB
// With i18next, the bundled JavaScript is 47KB
//
// The difference in size is mostly due to the fact that
// with Lisan, most of work is done AOT, resulting in
// translations being plain function calls
fn lisan(c: &mut Criterion) {
    let mut group = c.benchmark_group("i18n-lisan");

    group.bench_function("i18n-lisan", |b| {
        let mut runner = Runner::default();
        let json: serde_json::Value =
            serde_json::from_str(include_str!("./i18n-lisan/src/input.json")).unwrap();
        runner.set_input(&rmp_serde::to_vec(&json).unwrap());
        let wasm = include_bytes!("./i18n-lisan/build/bench.wasm");
        let module = runner.build_module(wasm);
        b.iter(|| runner.exec_module(&module));
    });

    group.finish();
}

fn i18n_next(c: &mut Criterion) {
    let mut group = c.benchmark_group("i18n-next");

    group.bench_function("i18n-next", |b| {
        let mut runner = Runner::default();
        let json: serde_json::Value =
            serde_json::from_str(include_str!("./i18n-next/src/input.json")).unwrap();
        runner.set_input(&rmp_serde::to_vec(&json).unwrap());
        let wasm = include_bytes!("./i18n-next/build/bench.wasm");
        let module = runner.build_module(wasm);
        b.iter(|| runner.exec_module(&module));
    });

    group.finish();
}

criterion_group!(benches, lisan, i18n_next);
criterion_main!(benches);
