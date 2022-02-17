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
            .get_typed_func::<(), (), _>(&mut self.store, "_start")
            .unwrap();
        main.call(&mut self.store, ()).unwrap();
    }

    // Instantiates and executes a module
    pub fn exec_module(&mut self, module: &Module) {
        let instance = self.linker.instantiate(&mut self.store, module).unwrap();
        let main = instance
            .get_typed_func::<(), (), _>(&mut self.store, "_start")
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
}
