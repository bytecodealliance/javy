use criterion::{criterion_group, criterion_main, Criterion};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::Wasi;
use wasmtime::*;

fn store_from_config() -> Store {
    let mut config = Config::new();

    config.cranelift_opt_level(OptLevel::SpeedAndSize);

    Wasi::add_to_config(&mut config);
    Store::new(&Engine::new(&config).unwrap())
}

fn linker(store: &Store) -> Linker {
    let wasi_ctx_builder = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();

    assert!(Wasi::set_context(&store, wasi_ctx_builder.unwrap()).is_ok());

    return Linker::new(store);
}

fn exec(linker: &Linker, module: &Module) {
    let instance = linker.instantiate(&module).unwrap();
    let run = instance.get_func("shopify_main").unwrap();
    let result = run.call(&[]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].i32(), Some(1179));
}


fn quickjs_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("qjs wasmtime");
    group.bench_function("control", |b| {
        let store = store_from_config();
        let linker = linker(&store);
        let bytes = &include_bytes!("javy.control.wasm");
        let compiled = store.engine().precompile_module(*bytes).unwrap();
        let module = Module::from_binary(store.engine(), &compiled).unwrap();

        b.iter(|| exec(&linker, &module))
    });

    group.bench_function("wizer", |b| {
        let store = store_from_config();
        let linker = linker(&store);
        let bytes = &include_bytes!("javy.opt.wizer.wasm");
        let compiled = store.engine().precompile_module(*bytes).unwrap();
        let module = Module::from_binary(store.engine(), &compiled).unwrap();

        b.iter(|| exec(&linker, &module))
    });

    group.finish();
}

criterion_group!(benches, quickjs_startup);
criterion_main!(benches);

