use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use wasmtime::{Caller, Config, Engine, Linker, Module, OptLevel, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::WasiCtx;
use wasi_common::sched::WasiSched;
use wasi_common::Poll;

struct CustomScheduler;

// #[async_trait::async_trait]
// impl WasiSched for CustomScheduler {
//     async fn poll_oneoff<'a>(&self, poll: &mut Poll<'a>) -> anyhow::Result<()> {
//         println!("==> poll_oneoff");
//         wasmtime_wasi::tokio::sched::poll_oneoff(poll).await
//     }

//     async fn sched_yield(&self) -> anyhow::Result<()> {
//         println!("==> yield");
//         tokio::task::yield_now().await;
//         Ok(())
//     }

//     async fn sleep(&self, duration: std::time::Duration) -> anyhow::Result<()> {
//         println!("==> sleep");
//         tokio::time::sleep(duration).await;
//         Ok(())
//     }
// }

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
        let random = wasmtime_wasi::tokio::random_ctx();
        let clocks = wasmtime_wasi::tokio::clocks_ctx();
        let sched = wasmtime_wasi::tokio::sched::sched_ctx();//CustomScheduler;
        let table = wasi_common::table::Table::new();

        let stdout = wasmtime_wasi::tokio::stdio::stdout();
        let mut wasi = WasiCtx::new(random, clocks, sched, table);
        wasi.set_stdout(Box::new(stdout));

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

    pub async fn exec(&mut self, input: Vec<u8>) -> Result<Vec<u8>> {
        let mut store = Store::new(self.linker.engine(), StoreContext::new(input));
        store.out_of_fuel_async_yield(u64::MAX, 10000);

        let module = Module::from_binary(self.linker.engine(), &self.wasm)?;

        let instance = self.linker.instantiate_async(&mut store, &module).await?;
        let run = instance.get_typed_func::<(), (), _>(&mut store, "shopify_main")?;

        run.call_async(&mut store, ()).await?;

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
    config.async_support(true);
    config.consume_fuel(true);

    Engine::new(&config).expect("failed to create engine")
}

fn setup_linker(engine: &Engine) -> Linker<StoreContext> {
    let mut linker = Linker::new(engine);

     wasmtime_wasi::tokio::add_to_linker(&mut linker, |ctx: &mut StoreContext| &mut ctx.wasi)
        .expect("failed to add wasi context");

    linker
        .func_wrap(
            "shopify_v1",
            "input_len",
            |mut caller: Caller<'_, StoreContext>, offset: i32| -> i32 {
                let len = caller.data().input.len();
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                mem.write(caller, offset as usize, &len.to_ne_bytes())
                    .unwrap();

                0
            },
        )
        .expect("failed to define input_len");

    linker
        .func_wrap(
            "shopify_v1",
            "input_copy",
            |mut caller: Caller<'_, StoreContext>, offset: i32| -> i32 {
                let input = caller.data().input.clone(); // TODO: avoid this copy
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                mem.write(caller, offset as usize, input.as_slice())
                    .unwrap();

                0
            },
        )
        .expect("failed to define input_copy");

    linker
        .func_wrap(
            "shopify_v1",
            "output_copy",
            |mut caller: Caller<'_, StoreContext>, offset: i32, len: i32| -> i32 {
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let mut buf = vec![0; len as usize];
                mem.read(&mut caller, offset as usize, buf.as_mut_slice())
                    .unwrap();

                caller.data_mut().output.resize(buf.len(), 0);
                caller.data_mut().output.copy_from_slice(buf.as_slice());

                0
            },
        )
        .expect("failed to define output_copy");

    // linker
    //     .func_wrap(
    //         "shopify_v1",
    //         "sock_connect",
    //         |mut caller: Caller<'_, StoreContext>| -> Result<(i32, i32)> {
    //             Ok((0, 0))
    //         },
    //     ).expect("failed to define sock_connect");

    // linker
    //     .func_wrap(
    //         "shopify_v1",
    //         "sock_recv",
    //         |mut caller: Caller<'_, StoreContext>, fd: i32, ri_data: i32, ri_flags: i32| -> Result<(i32, i32)> {
    //             Ok((0, 0))
    //         },
    //     ).expect("failed to define sock_recv");

    //  linker
    //     .func_wrap(
    //         "shopify_v1",
    //         "sock_send",
    //         |mut caller: Caller<'_, StoreContext>, fd: i32, si_data: i32, si_flags: i32| -> Result<i32> {
    //             Ok((0, 0))
    //         },
    //     ).expect("failed to define sock_send");

    // linker
    //     .func_wrap(
    //         "shopify_v1",
    //         "sock_shutdown",
    //         |mut caller: Caller<'_, StoreContext>, fd: i32, how: i32| -> Result<()> {
    //             Ok((0, 0))
    //         }
    //     ).expect("failed to define sock_shutdown");


    linker
}
