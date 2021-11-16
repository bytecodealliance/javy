mod runner;
use crate::runner::Runner;
use criterion::{criterion_group, criterion_main, Criterion};
use wizer::Wizer;

// 242us - 249us
// The cost of creating a QuickJS runtime and context
fn startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("quickjs");
    group.bench_function("startup", |b| {
        let wasm = include_bytes!("./quickjs-startup/quickjs_startup.wasm");
        let mut runner = Runner::default();
        let module = runner.build_module(wasm);
        let instance = runner.instantiate(&module);
        b.iter(|| runner.exec_instance(&instance));
    });
    group.finish();
}

// 7.6us - 8.1us
// This is the simplest script that we can ever have:
// ```
// var Shopify = {
//   main: function() {
//   }
// };
fn eval_noop(c: &mut Criterion) {
    let mut group = c.benchmark_group("quickjs");
    group.bench_function("eval noop", |b| {
        let mut runner = Runner::default();
        let script = include_str!("./js-scripts/noop.js");
        let wasm = include_bytes!("./quickjs-eval/quickjs_eval.wasm");
        let wizened = wizen(wasm, script);
        let module = runner.build_module(&wizened);
        let instance = runner.instantiate(&module);
        b.iter(|| runner.exec_instance(&instance));
    });
    group.finish();
}

// 672us - 694us
// This is a more involved script. Contains the lisan (i18n framework)
// as a dependency
fn eval_lisan(c: &mut Criterion) {
    let mut group = c.benchmark_group("quickjs");
    group.bench_function("eval lisan", |b| {
        let mut runner = Runner::default();
        let script = include_str!("./js-scripts/lisan.js");
        let wasm = include_bytes!("./quickjs-eval/quickjs_eval.wasm");
        let wizened = wizen(wasm, script);
        let module = runner.build_module(&wizened);
        let instance = runner.instantiate(&module);
        b.iter(|| runner.exec_instance(&instance));
    });
    group.finish();
}

fn wizen(wasm: &[u8], script: &str) -> Vec<u8> {
    std::env::set_var("SCRIPT", script);
    let result = Wizer::new()
        .allow_wasi(true)
        .inherit_env(true)
        .run(wasm)
        .unwrap();
    std::env::remove_var("SCRIPT");

    result
}

// criterion_group!(benches, startup, eval_noop, eval_lisan);
criterion_group!(benches, eval_lisan);
criterion_main!(benches);
