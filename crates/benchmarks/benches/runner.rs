use wasmtime::*;
use wasmtime_wasi::{sync, WasiCtx, WasiCtxBuilder};

struct Context {
    wasi: WasiCtx,
    input: Vec<u8>,
}

impl Context {
    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input = input;
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
            input: vec![],
        }
    }
}

pub struct Runner {
    linker: Linker<Context>,
    store: Store<Context>,
}

impl Default for Runner {
    fn default() -> Self {
        let context = Context::default();
        let mut store = create_store(context);
        let linker = create_linker(&mut store);

        Self { linker, store }
    }
}

impl Runner {
    pub fn build_module(&mut self, wasm: &[u8]) -> Module {
        Module::from_binary(self.linker.engine(), wasm).unwrap()
    }

    pub fn set_input(&mut self, input: &[u8]) {
        self.store.data_mut().set_input(input.into());
    }

    pub fn instantiate(&mut self, module: &Module) -> Instance {
        self.linker.instantiate(&mut self.store, module).unwrap()
    }

    // Executes an instance
    pub fn exec_instance(&mut self, instance: &Instance) {
        let main = instance
            .get_typed_func::<(), (), _>(&mut self.store, "shopify_main")
            .unwrap();
        main.call(&mut self.store, ()).unwrap();
    }

    // Instantiates and executes a module
    pub fn exec_module(&mut self, module: &Module) {
        let instance = self.linker.instantiate(&mut self.store, module).unwrap();
        let main = instance
            .get_typed_func::<(), (), _>(&mut self.store, "shopify_main")
            .unwrap();
        main.call(&mut self.store, ()).unwrap();
    }
}

fn create_store(data: Context) -> Store<Context> {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::SpeedAndSize);

    Store::new(&Engine::new(&config).unwrap(), data)
}

fn create_linker(store: &mut Store<Context>) -> Linker<Context> {
    let mut linker = Linker::new(store.engine());
    sync::add_to_linker(&mut linker, |d: &mut Context| &mut d.wasi).unwrap();

    linker
        .func_wrap(
            "shopify_v1",
            "input_len",
            move |mut caller: Caller<'_, Context>, offset: u32| -> u32 {
                let memory = caller
                    .get_export("memory")
                    .and_then(|slot| slot.into_memory())
                    .expect("Couldn't get access to caller's memory");

                let data = caller.data();
                let len = data.input.len();
                memory
                    .write(caller.as_context_mut(), offset as usize, &len.to_ne_bytes())
                    .expect("Couldn't write input length");

                0
            },
        )
        .expect("Could not define input_len import");

    linker
        .func_wrap(
            "shopify_v1",
            "input_copy",
            move |mut caller: Caller<'_, Context>, offset: u32| -> u32 {
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

                0
            },
        )
        .expect("Could not define input_copy import");

    linker
        .func_wrap(
            "shopify_v1",
            "output_copy",
            move |mut caller: Caller<'_, Context>, offset: u32, len: u32| -> u32 {
                let mem = caller
                    .get_export("memory")
                    .and_then(|slot| slot.into_memory())
                    .expect("Couldn't get access to caller's memory");

                let mut buf = vec![0; len as usize];
                mem.read(caller.as_context_mut(), offset as usize, buf.as_mut_slice())
                    .expect("Couldn't read output from module memory");
                assert!(buf.len() > 0);

                0
            },
        )
        .expect("Could not define output_copy import");

    linker
}
