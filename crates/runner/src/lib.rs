use anyhow::{bail, Result};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{cmp, fs};
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasi_common::sync::WasiCtxBuilder;
use wasi_common::WasiCtx;
use wasmtime::{Config, Engine, Linker, Module, OptLevel, Store};

pub struct Builder {
    /// The JS source.
    input: PathBuf,
    /// Root path. Used resolve the absolute path of the JS source.
    root: PathBuf,
    /// `javy` binary path.
    bin_path: String,
    /// The path to the wit file.
    wit: Option<PathBuf>,
    /// The name of the wit world.
    world: Option<String>,
    /// The logger capacity, in bytes.
    capacity: usize,
    built: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            capacity: usize::MAX,
            input: PathBuf::from("identity.js"),
            wit: None,
            world: None,
            bin_path: "javy".into(),
            root: Default::default(),
            built: false,
        }
    }
}

impl Builder {
    pub fn root(&mut self, root: impl Into<PathBuf>) -> &mut Self {
        self.root = root.into();
        self
    }

    pub fn input(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.input = path.into();
        self
    }

    pub fn bin(&mut self, bin: impl Into<String>) -> &mut Self {
        self.bin_path = bin.into();
        self
    }

    pub fn wit(&mut self, wit: impl Into<PathBuf>) -> &mut Self {
        self.wit = Some(wit.into());
        self
    }

    pub fn world(&mut self, world: impl Into<String>) -> &mut Self {
        self.world = Some(world.into());
        self
    }

    pub fn build(&mut self) -> Result<Runner> {
        if self.built {
            bail!("Builder already used to build a runner")
        }

        if (self.wit.is_some() && self.world.is_none())
            || (self.wit.is_none() && self.world.is_some())
        {
            bail!("Both `wit` and `world` must be defined")
        }

        let Self {
            bin_path,
            input,
            wit,
            world,
            capacity,
            root,
            built: _,
        } = std::mem::take(self);

        self.built = true;

        Ok(Runner::new(bin_path, root, input, wit, world, capacity))
    }
}

pub struct Runner {
    pub wasm: Vec<u8>,
    linker: Linker<StoreContext>,
    capacity: usize,
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

impl Runner {
    fn new(
        bin: String,
        root: PathBuf,
        source: impl AsRef<Path>,
        wit: Option<PathBuf>,
        world: Option<String>,
        capacity: usize,
    ) -> Self {
        let wasm_file_name = format!("{}.wasm", uuid::Uuid::new_v4());

        // This directory is unique and will automatically get deleted
        // when `tempdir` goes out of scope.
        let Ok(tempdir) = tempfile::tempdir() else {
            panic!("Could not create temporary directory for .wasm test artifacts");
        };
        let wasm_file = tempdir.path().join(wasm_file_name);
        let js_file = root.join(source);
        let wit_file = wit.map(|p| root.join(p));

        let mut args = vec![
            "compile".to_string(),
            js_file.to_str().unwrap().to_string(),
            "-o".to_string(),
            wasm_file.to_str().unwrap().to_string(),
        ];

        if let (Some(wit_file), Some(world)) = (wit_file, world) {
            args.push("--wit".to_string());
            args.push(wit_file.to_str().unwrap().to_string());
            args.push("-n".to_string());
            args.push(world.to_string());
        }

        let output = Command::new(bin)
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
            capacity,
        }
    }

    pub fn exec(&mut self, input: &[u8]) -> Result<(Vec<u8>, Vec<u8>, u64)> {
        self.exec_func("_start", input)
    }

    pub fn exec_func(&mut self, func: &str, input: &[u8]) -> Result<(Vec<u8>, Vec<u8>, u64)> {
        let mut store = Store::new(
            self.linker.engine(),
            StoreContext::new(input, self.capacity),
        );
        const INITIAL_FUEL: u64 = u64::MAX;
        store.set_fuel(INITIAL_FUEL)?;

        let module = Module::from_binary(self.linker.engine(), &self.wasm)?;

        let instance = self.linker.instantiate(&mut store, &module)?;
        let run = instance.get_typed_func::<(), ()>(&mut store, func)?;

        let res = run.call(&mut store, ());
        let fuel_consumed = INITIAL_FUEL - store.get_fuel()?;
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

    wasi_common::sync::add_to_linker(&mut linker, |ctx: &mut StoreContext| &mut ctx.wasi)
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

#[cfg(test)]
mod tests {
    use super::Builder;
    use anyhow::Result;

    #[test]
    fn test_validation_on_world_defined() -> Result<()> {
        let result = Builder::default().world("foo").build();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_validation_on_wit_defined() -> Result<()> {
        let result = Builder::default().wit("foo.wit").build();

        assert!(result.is_err());
        Ok(())
    }
}
