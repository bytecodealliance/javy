use anyhow::Result;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{cmp, fs};
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasmtime::{Config, Engine, Linker, Module, OptLevel, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::WasiCtx;

pub struct Runner {
    pub wasm: Vec<u8>,
    linker: Linker<StoreContext>,
    log_capacity: usize,
}

#[derive(Debug)]
pub struct RunnerError {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub err: anyhow::Error,
}

impl Error for RunnerError {}

impl Display for RunnerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error: {:?}, stdout: {:?}, stderr: {:?}",
            self.err, self.stdout, self.stderr
        )
    }
}

struct StoreContext {
    wasi_output: WritePipe<Cursor<Vec<u8>>>,
    wasi: WasiCtx,
    log_stream: WritePipe<LogWriter>,
}

impl StoreContext {
    fn new(input: &[u8], capacity: usize) -> Self {
        let wasi_output = WritePipe::new_in_memory();
        let log_stream = WritePipe::new(LogWriter::new(capacity));
        let wasi = WasiCtxBuilder::new()
            .stdout(Box::new(wasi_output.clone()))
            .stdin(Box::new(ReadPipe::from(input)))
            .stderr(Box::new(log_stream.clone()))
            .build();
        Self {
            wasi,
            wasi_output,
            log_stream,
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
        Self::new_with_fixed_logging_capacity(js_file, None, None, usize::MAX)
    }

    pub fn new_with_exports(
        js_file: impl AsRef<Path>,
        wit_path: impl AsRef<Path>,
        world: &str,
    ) -> Self {
        Self::new_with_fixed_logging_capacity(
            js_file,
            Some(wit_path.as_ref()),
            Some(world),
            usize::MAX,
        )
    }

    fn new_with_fixed_logging_capacity(
        js_file: impl AsRef<Path>,
        wit_path: Option<&Path>,
        wit_world: Option<&str>,
        capacity: usize,
    ) -> Self {
        let wasm_file_name = format!("{}.wasm", uuid::Uuid::new_v4());

        let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let sample_scripts = root.join("tests").join("sample-scripts");
        // This directory is unique and will automatically get deleted
        // when `tempdir` goes out of scope.
        let Ok(tempdir) = tempfile::tempdir() else {
            panic!("Could not create temporary directory for .wasm test artifacts");
        };
        let wasm_file = tempdir.path().join(wasm_file_name);
        let js_file = sample_scripts.join(js_file);
        let wit_file = wit_path.map(|p| sample_scripts.join(p));

        let mut args = vec![
            "compile".to_string(),
            js_file.to_str().unwrap().to_string(),
            "-o".to_string(),
            wasm_file.to_str().unwrap().to_string(),
        ];

        if let (Some(wit_file), Some(world)) = (wit_file, wit_world) {
            args.push("--wit".to_string());
            args.push(wit_file.to_str().unwrap().to_string());
            args.push("-n".to_string());
            args.push(world.to_string());
        }

        let output = Command::new(env!("CARGO_BIN_EXE_javy"))
            .current_dir(root)
            .args(args)
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

        Self {
            wasm,
            linker,
            log_capacity: capacity,
        }
    }

    pub fn exec(&mut self, input: &[u8]) -> Result<(Vec<u8>, Vec<u8>, u64)> {
        self.exec_func("_start", input)
    }

    pub fn exec_func(&mut self, func: &str, input: &[u8]) -> Result<(Vec<u8>, Vec<u8>, u64)> {
        let mut store = Store::new(
            self.linker.engine(),
            StoreContext::new(input, self.log_capacity),
        );
        store.add_fuel(u64::MAX)?;

        let module = Module::from_binary(self.linker.engine(), &self.wasm)?;

        let instance = self.linker.instantiate(&mut store, &module)?;
        let run = instance.get_typed_func::<(), ()>(&mut store, func)?;

        let res = run.call(&mut store, ());
        let fuel_consumed = store.fuel_consumed().unwrap();
        let store_context = store.into_data();
        drop(store_context.wasi);
        let logs = store_context
            .log_stream
            .try_into_inner()
            .expect("log stream reference still exists")
            .buffer;
        let output = store_context
            .wasi_output
            .try_into_inner()
            .expect("Output stream reference still exists")
            .into_inner();

        match res {
            Ok(_) => Ok((output, logs, fuel_consumed)),
            Err(err) => Err(RunnerError {
                stdout: output,
                stderr: logs,
                err,
            }
            .into()),
        }
    }
}

fn setup_engine() -> Engine {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::SpeedAndSize);
    config.consume_fuel(true);
    Engine::new(&config).expect("failed to create engine")
}

fn setup_linker(engine: &Engine) -> Linker<StoreContext> {
    let mut linker = Linker::new(engine);

    wasmtime_wasi::sync::add_to_linker(&mut linker, |ctx: &mut StoreContext| &mut ctx.wasi)
        .expect("failed to add wasi context");

    linker
}

#[derive(Debug)]
pub struct LogWriter {
    pub buffer: Vec<u8>,
    capacity: usize,
}

impl LogWriter {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: Default::default(),
            capacity,
        }
    }
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let available_capacity = self.capacity - self.buffer.len();
        let amount_to_take = cmp::min(available_capacity, buf.len());
        self.buffer.extend_from_slice(&buf[..amount_to_take]);
        Ok(amount_to_take)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
