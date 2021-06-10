use criterion::{criterion_group, criterion_main, Criterion};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::Wasi;
use wasmtime::*;

fn store_from_config(alter_limits: bool) -> Store {
    let mut config = Config::new();

    config.cranelift_opt_level(OptLevel::SpeedAndSize);

    if alter_limits {
        config
            .max_instances(1000000)
            .max_tables(1000000)
            .max_memories(1000000)
            .static_memory_maximum_size(0);
    }

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
        let store = store_from_config(false);
        let linker = linker(&store);
        let module = Module::new(store.engine(), &include_bytes!("javy.control.wasm")).unwrap();

        b.iter(|| exec(&linker, &module))
    });

    group.bench_function("wizer", |b| {
        let store = store_from_config(false);
        let linker = linker(&store);
        let module = Module::new(store.engine(), &include_bytes!("javy.opt.wizer.wasm")).unwrap();

        b.iter(|| exec(&linker, &module))
    });

    group.finish();
}

criterion_group!(benches, quickjs_startup);
criterion_main!(benches);

