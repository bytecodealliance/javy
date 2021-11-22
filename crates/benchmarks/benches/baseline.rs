mod runner;

use crate::runner::Runner;
use criterion::{criterion_group, criterion_main, Criterion};

fn baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("baseline");

    group.bench_function("baseline", |b| {
        let mut runner = Runner::default();
        let json: serde_json::Value =
            serde_json::from_str(include_str!("./default/src/input.json")).unwrap();
        runner.set_input(&rmp_serde::to_vec(&json).unwrap());
        let wasm = include_bytes!("./default/build/bench.wasm");
        let module = runner.build_module(wasm);
        b.iter(|| runner.exec_module(&module));
    });

    group.finish();
}

criterion_group!(benches, baseline);
criterion_main!(benches);
