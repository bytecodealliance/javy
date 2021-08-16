use criterion::{criterion_group, criterion_main, Criterion};
use wasmtime::*;
use wasmtime_wasi::{sync, WasiCtx, WasiCtxBuilder};

struct Data {
    wasi: WasiCtx,
    input: Vec<u8>,
}

impl Data {
    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input = input;
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
            input: vec![],
        }
    }
}

fn assemblyscript_input() -> Vec<u8> {
    let json: serde_json::Value = serde_json::from_str(include_str!("./as_input.json")).unwrap();
    rmp_serde::to_vec(&json).unwrap()
}

fn javascript_input() -> Vec<u8> {
    let json: serde_json::Value = serde_json::from_str(include_str!("./js_input.json")).unwrap();
    rmp_serde::to_vec(&json).unwrap()
}

fn make_store(data: Data) -> Store<Data> {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::SpeedAndSize);

    Store::new(&Engine::new(&config).unwrap(), data)
}

fn make_linker(store: &mut Store<Data>) -> Linker<Data> {
    let mut linker = Linker::new(store.engine());
    sync::add_to_linker(&mut linker, |d: &mut Data| &mut d.wasi).unwrap();

    linker
        .func_wrap(
            "shopify_v1",
            "input_len",
            move |mut caller: Caller<'_, Data>, offset: i32| {
                let memory = caller
                    .get_export("memory")
                    .and_then(|slot| slot.into_memory())
                    .expect("Couldn't get access to caller's memory");

                let data = caller.data();
                let len = data.input.len();
                memory
                    .write(caller.as_context_mut(), offset as usize, &len.to_ne_bytes())
                    .expect("Couldn't write input length");
            },
        )
        .expect("Could not define input_len import");

    linker
        .func_wrap(
            "shopify_v1",
            "input_copy",
            move |mut caller: Caller<'_, Data>, offset: i32| {
                let memory = caller
                    .get_export("memory")
                    .and_then(|slot| slot.into_memory())
                    .expect("Couldn't get access to caller's memory");

                let (backing_memory, data) = memory.data_and_store_mut(&mut caller);
                let offset = offset as usize;
                let slot = backing_memory
                    .get_mut(offset..offset + data.input.len())
                    .expect("Couldn't allocate memory space to copy input");
                slot.copy_from_slice(&data.input);
            },
        )
        .expect("Could not define input_copy import");

    linker
        .func_wrap(
            "shopify_v1",
            "output_copy",
            move |mut caller: Caller<'_, Data>, offset: i32, len: i32| {
                let mem = caller
                    .get_export("memory")
                    .and_then(|slot| slot.into_memory())
                    .expect("Couldn't get access to caller's memory");

                let mut buf = vec![0; len as usize];
                mem.read(caller.as_context_mut(), offset as usize, buf.as_mut_slice())
                    .expect("Couldn't read output from module memory");
                assert!(buf.len() > 0);
            },
        )
        .expect("Could not define output_copy import");

    linker
}

fn exec(store: &mut Store<Data>, linker: &Linker<Data>, module: &Module) {
    let instance = linker.instantiate(&mut *store, &module).unwrap();
    let run = instance
        .get_typed_func::<(), (), _>(&mut *store, "shopify_main")
        .unwrap();
    run.call(&mut *store, ()).unwrap();
}

fn js(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasmtime");

    group.bench_function("js", |b| {
        let mut data = Data::default();
        data.set_input(javascript_input());

        let mut store = make_store(data);
        let linker = make_linker(&mut store);
        let bytes = &include_bytes!("js.wasm");
        let module = Module::from_binary(store.engine(), *bytes).unwrap();

        b.iter(|| exec(&mut store, &linker, &module))
    });

    group.bench_function("as", |b| {
        let mut data = Data::default();
        data.set_input(assemblyscript_input());

        let mut store = make_store(data);
        let linker = make_linker(&mut store);
        let bytes = &include_bytes!("as.wasm");
        let module = Module::from_binary(store.engine(), *bytes).unwrap();

        b.iter(|| exec(&mut store, &linker, &module))
    });

    group.finish();
}

criterion_group!(benches, js);
criterion_main!(benches);
