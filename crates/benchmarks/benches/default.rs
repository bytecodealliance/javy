mod runner;

use crate::runner::Runner;
use criterion::{criterion_group, criterion_main, Criterion};

fn default(c: &mut Criterion) {
    let mut group = c.benchmark_group("default");

    group.bench_function("default", |b| {
        let mut runner = Runner::default();
        let json: serde_json::Value = serde_json::from_str(include_str!("./default/src/input.json")).unwrap();
        runner.set_input(&rmp_serde::to_vec(&json).unwrap());
        let wasm = include_bytes!("./default/build/index.wasm");
        let module = runner.build_module(wasm);
        b.iter(|| runner.exec(&module));
    });

    group.finish();
}

criterion_group!(benches, default);
criterion_main!(benches);
