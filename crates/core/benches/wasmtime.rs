use criterion::{criterion_group, criterion_main, Criterion};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::Wasi;

fn store_from_config() -> Store {
    let mut config = Config::new();

    config.cranelift_opt_level(OptLevel::SpeedAndSize);

    Wasi::add_to_config(&mut config);
    Store::new(&Engine::new(&config).unwrap())
}

fn assemblyscript() -> Vec<u8> {
    let json: serde_json::Value = serde_json::from_str(include_str!("./as_input.json")).unwrap();
    rmp_serde::to_vec(&json).unwrap()
}

fn javascript() -> Vec<u8> {
    let json: serde_json::Value = serde_json::from_str(include_str!("./js_input.json")).unwrap();
    rmp_serde::to_vec(&json).unwrap()
}

fn linker(store: &Store) -> Linker {
    let wasi_ctx_builder = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();

    assert!(Wasi::set_context(&store, wasi_ctx_builder).is_ok());

    let mut linker = Linker::new(store);

    linker
        .func(
            "shopify_v1",
            "input_len",
            move |caller: Caller, offset: i32| {
                let data = caller.store().get::<Vec<u8>>().unwrap();
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let len = data.len();
                mem.write(offset as usize, &len.to_ne_bytes())
                    .expect("Couldn't write input length to module memory");
            },
        )
        .expect("Could not define input_len import");

    linker
        .func(
            "shopify_v1",
            "input_copy",
            move |caller: Caller, offset: i32| {
                let data = caller.store().get::<Vec<u8>>().unwrap();
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let input = &data;
                mem.write(offset as usize, input.as_ref())
                    .expect("Couldn't write input to module memory");
            },
        )
        .expect("Could not define input_copy import");

    linker
        .func(
            "shopify_v1",
            "output_copy",
            move |caller: Caller, offset: i32, len: i32| {
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let mut buf = vec![0; len as usize];
                mem.read(offset as usize, buf.as_mut_slice())
                    .expect("Couldn't read output from module memory");
                assert!(buf.len() > 0);
            },
        )
        .expect("Could not define output_copy import");

    linker
}

fn exec(linker: &Linker, module: &Module) {
    let instance = linker.instantiate(&module).unwrap();
    let run = instance.get_typed_func::<(), ()>("shopify_main").unwrap();
    run.call(()).unwrap();
}

fn js(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasmtime");

    group.bench_function("js", |b| {
        let store = store_from_config();
        store.set(javascript()).unwrap();
        let linker = linker(&store);
        let bytes = &include_bytes!("js.wasm");
        let module = Module::from_binary(store.engine(), *bytes).unwrap();

        b.iter(|| exec(&linker, &module))
    });

    group.bench_function("as", |b| {
        let store = store_from_config();
        store.set(assemblyscript()).unwrap();
        let linker = linker(&store);
        let bytes = &include_bytes!("as.wasm");
        let module = Module::from_binary(store.engine(), *bytes).unwrap();

        b.iter(|| exec(&linker, &module))
    });

    group.finish();
}

criterion_group!(benches, js);
criterion_main!(benches);
