use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::Wasi;
use wasmtime::*;

fn store_from_config() -> Store {
    let mut config = Config::default();
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
    let run = instance.get_func("run").unwrap();
    run.call(&[]);
}

fn quickjs_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("js startup");
    group.bench_function("control", |b| {
        let store = store_from_config();
        let linker = linker(&store);
        let module = Module::new(store.engine(), &include_bytes!("javy.control.wasm")).unwrap();

        b.iter(|| exec(&linker, &module))
    });

    group.bench_function("wizer", |b| {
        let store = store_from_config();
        let linker = linker(&store);
        let module = Module::new(store.engine(), &include_bytes!("javy.wizer.wasm")).unwrap();

        b.iter(|| exec(&linker, &module))
    });

}

criterion_group!(benches, quickjs_startup);
criterion_main!(benches);

