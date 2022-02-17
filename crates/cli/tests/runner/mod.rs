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

impl Default for Runner {
    fn default() -> Self {
        Self::new("identity.js")
    }
}

impl Runner {
    pub fn new(js_file: impl AsRef<Path>) -> Self {
        let wasm_file_name = format!("{}.wasm", uuid::Uuid::new_v4());

        let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let wasm_file = std::env::temp_dir().join(wasm_file_name);
        let js_file = root.join("tests").join("sample-scripts").join(js_file);

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
        let run = instance.get_typed_func::<(), (), _>(&mut store, "_start")?;

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
}
