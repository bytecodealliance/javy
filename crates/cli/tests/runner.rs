use wasmtime::{Linker, Store, Module, Caller, Engine, OptLevel, Config};
use wasmtime_wasi::{sync::WasiCtxBuilder, Wasi};
use std::fs;
use std::process::Command;
use anyhow::{Result, bail};
use std::path::PathBuf;
use std::cell::RefCell;

pub struct Runner{
    wasm: Vec<u8>,
    linker: Linker,
    store: Store
}

#[derive(Debug, Clone)]
struct StoreContext {
    input: RefCell<Vec<u8>>,
    input_len: RefCell<usize>,
    output: RefCell<Vec<u8>>,
}

impl Runner {
    pub fn new() -> Result<Self> {
        let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
        let js_file = root_dir.join("tests").join("index.js");
        let wasm_file = root_dir.join("tests").join("index.wasm");

        let output = Command::new(env!("CARGO_BIN_EXE_javy"))
            .arg(&js_file)
            .arg("-o")
            .arg(&wasm_file)
            .output()?;

        if !output.status.success() {
            bail!("Couldn't create wasm binary");
        }

        let store = setup_store()?;
        let wasm = fs::read(&wasm_file)?;

        Ok(Self {
            wasm,
            store: store.clone(),
            linker: setup_linker(&store)?,
        })
    }

    pub fn define_imports(&mut self) -> Result<&mut Self> {
        self.linker
            .func(
                "shopify_v1",
                "input_len",
                move |caller: Caller, offset: i32| {
                    let context = caller.store().get::<StoreContext>().unwrap();
                    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                    let len = context.input_len.clone().into_inner();
                    mem.write(offset as usize, &len.to_ne_bytes()).unwrap();
                },
            )?;

        self.linker
            .func(
                "shopify_v1",
                "input_copy",
                move |caller: Caller, offset: i32| {
                    let context = caller.store().get::<StoreContext>().unwrap();
                    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                    mem.write(offset as usize, &context.input.clone().into_inner()).unwrap();
                },
            )?;

        self.linker
            .func(
                "shopify_v1",
                "output_copy",
                move |caller: Caller, offset: i32, len: i32| {
                    let context = caller.store().get::<StoreContext>().unwrap();
                    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                    let mut buf = vec![0; len as usize];
                    mem.read(offset as usize, buf.as_mut_slice()).unwrap();

                    context.with_output(buf.to_vec());
                },
            )?;

        self.store.set(StoreContext::default()).unwrap();


        Ok(self)
    }

    pub fn exec(&mut self, input: Vec<u8>) -> Result<Vec<u8>> {
        let context = self.store.get::<StoreContext>().unwrap();
        context.with_input(input);

        let module = Module::from_binary(&self.store.engine(), &self.wasm)?;
        let instance = self.linker.instantiate(&module)?;
        let run = instance.get_typed_func::<(), ()>("shopify_main")?;

        run.call(())?;
        let context = self.store.get::<StoreContext>().unwrap();
        Ok(context.output.clone().into_inner())
    }
}

impl StoreContext {
    pub fn default() -> Self {
        Self {
            input: RefCell::new(vec![]),
            input_len: RefCell::new(0),
            output: RefCell::new(vec![]),
        }
    }

    pub fn with_output(&self, output: Vec<u8>) -> &Self {
        self.output.replace(output);
        self
    }

    pub fn with_input(&self, input: Vec<u8>) -> &Self {
        let len = input.len();
        self.input.replace(input);
        self.input_len.replace(len);
        self
    }
}

fn setup_store() -> Result<Store> {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::SpeedAndSize);
    Wasi::add_to_config(&mut config);
    Ok(Store::new(&Engine::new(&config)?))
}

fn setup_linker(store: &Store) -> Result<Linker> {
    let wasi_ctx_builder = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();

    assert!(Wasi::set_context(&store, wasi_ctx_builder).is_ok());

    Ok(Linker::new(store))
}
