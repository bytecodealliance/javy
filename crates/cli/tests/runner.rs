use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use wasmtime::{Caller, Config, Engine, Linker, Module, OptLevel, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::WasiCtx;

pub struct Runner {
    wasm: Vec<u8>,
    linker: Linker<StoreContext>,
}

struct StoreContext {
    input: Vec<u8>,
    output: Vec<u8>,
    wasi: WasiCtx,
}

impl Default for StoreContext {
    fn default() -> Self {
        let wasi = WasiCtxBuilder::new().inherit_stdio().build();

        Self {
            wasi,
            input: Vec::default(),
            output: Vec::default(),
        }
    }
}

fn root_dir() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .expect("failed to get current cargo dir")
        .into()
}

impl Default for Runner {
    fn default() -> Self {
        Self::new("identity.js")
    }
}

impl Runner {
    fn new(js_file: impl AsRef<Path>) -> Self {
        let root = root_dir();
        let wasm_file = root.join("tests").join("target").join("out.wasm");

        let js_file = root.join("tests").join("fixtures").join(js_file);

        let output = Command::new(env!("CARGO_BIN_EXE_javy"))
            .current_dir(root)
            .arg(&js_file)
            .arg("-o")
            .arg(&wasm_file)
            .output()
            .expect("failed to run command");

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        if !output.status.success() {
            panic!("terminated with status = {}", output.status);
        }

        let wasm = fs::read(&wasm_file).expect("failed to read wasm module");

        let engine = setup_engine();
        let linker = setup_linker(&engine);

        Self { wasm, linker }
    }

    pub fn exec(&mut self, input: Vec<u8>) -> Result<Vec<u8>> {
        let mut store = Store::new(self.linker.engine(), StoreContext::new(input));

        let module = Module::from_binary(self.linker.engine(), &self.wasm)?;

        let instance = self.linker.instantiate(&mut store, &module)?;
        let run = instance.get_typed_func::<(), (), _>(&mut store, "shopify_main")?;

        run.call(&mut store, ())?;

        Ok(store.into_data().output)
    }
}

impl StoreContext {
    fn new(input: Vec<u8>) -> Self {
        Self {
            input,
            ..Default::default()
        }
    }
}

fn setup_engine() -> Engine {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::SpeedAndSize);
    Engine::new(&config).expect("failed to create engine")
}

fn setup_linker(engine: &Engine) -> Linker<StoreContext> {
    let mut linker = Linker::new(engine);

    wasmtime_wasi::sync::add_to_linker(&mut linker, |ctx: &mut StoreContext| &mut ctx.wasi)
        .expect("failed to add wasi context");

    linker
        .func_wrap(
            "shopify_v1",
            "input_len",
            move |mut caller: Caller<'_, StoreContext>, offset: i32| {
                let len = caller.data().input.len();
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                mem.write(caller, offset as usize, &len.to_ne_bytes())
                    .unwrap();
            },
        )
        .expect("failed to define input_len");

    linker
        .func_wrap(
            "shopify_v1",
            "input_copy",
            move |mut caller: Caller<'_, StoreContext>, offset: i32| {
                let input = caller.data().input.clone(); // TODO: avoid this copy
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                mem.write(caller, offset as usize, input.as_slice())
                    .unwrap();
            },
        )
        .expect("failed to define input_copy");

    linker
        .func_wrap(
            "shopify_v1",
            "output_copy",
            move |mut caller: Caller<'_, StoreContext>, offset: i32, len: i32| {
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let mut buf = vec![0; len as usize];
                mem.read(&mut caller, offset as usize, buf.as_mut_slice())
                    .unwrap();

                caller.data_mut().output.resize(buf.len(), 0);
                caller.data_mut().output.copy_from_slice(buf.as_slice());
            },
        )
        .expect("failed to define output_copy");

    linker
}
