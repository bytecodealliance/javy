use anyhow::{anyhow, bail, Result};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::{cmp, fs};
use tempfile::TempDir;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasi_common::sync::WasiCtxBuilder;
use wasi_common::WasiCtx;
use wasmtime::{AsContextMut, Config, Engine, Instance, Linker, Module, OptLevel, Store};

#[derive(Clone)]
pub enum JavyCommand {
    Build,
    Compile,
}

#[derive(Clone)]
pub enum Plugin {
    V2,
    Default,
    User,
    /// Pass the default plugin on the CLI as a user plugin.
    DefaultAsUser,
}

impl Plugin {
    pub fn namespace(&self) -> &'static str {
        match self {
            Self::V2 => "javy_quickjs_provider_v2",
            // Could try and derive this but not going to for now since tests
            // will break if it changes.
            Self::Default | Self::DefaultAsUser => "javy_quickjs_provider_v3",
            Self::User { .. } => "test_plugin",
        }
    }

    pub fn path(&self) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        match self {
            Self::V2 => root
                .join("..")
                .join("..")
                .join("crates")
                .join("cli")
                .join("src")
                .join("javy_quickjs_provider_v2.wasm"),
            Self::User => root.join("test_plugin.wasm"),
            Self::Default | Self::DefaultAsUser => root
                .join("..")
                .join("..")
                .join("target")
                .join("wasm32-wasip1")
                .join("release")
                .join("plugin_wizened.wasm"),
        }
    }
}

#[derive(Clone)]
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
    /// Whether to enable the `Javy.IO` builtins.
    javy_stream_io: Option<bool>,
    /// Whether to override JSON.parse and JSON.stringify with a SIMD based
    /// implementation.
    simd_json_builtins: Option<bool>,
    /// Whether to enable the `TextEncoder` and `TextDecoder` APIs.
    text_encoding: Option<bool>,
    /// Whether to enable the event loop.
    event_loop: Option<bool>,
    built: bool,
    /// Preload the module at path, using the given instance name.
    preload: Option<(String, PathBuf)>,
    /// Whether to use the `compile` or `build` command.
    command: JavyCommand,
    /// The javy plugin.
    /// Used for import validation purposes only.
    plugin: Plugin,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            input: PathBuf::from("identity.js"),
            wit: None,
            world: None,
            bin_path: "javy".into(),
            root: Default::default(),
            built: false,
            preload: None,
            command: JavyCommand::Build,
            javy_stream_io: None,
            simd_json_builtins: None,
            text_encoding: None,
            event_loop: None,
            plugin: Plugin::Default,
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

    pub fn preload(&mut self, ns: String, wasm: impl Into<PathBuf>) -> &mut Self {
        self.preload = Some((ns, wasm.into()));
        self
    }

    pub fn javy_stream_io(&mut self, enabled: bool) -> &mut Self {
        self.javy_stream_io = Some(enabled);
        self
    }

    pub fn simd_json_builtins(&mut self, enabled: bool) -> &mut Self {
        self.simd_json_builtins = Some(enabled);
        self
    }

    pub fn text_encoding(&mut self, enabled: bool) -> &mut Self {
        self.text_encoding = Some(enabled);
        self
    }

    pub fn event_loop(&mut self, enabled: bool) -> &mut Self {
        self.event_loop = Some(enabled);
        self
    }

    pub fn command(&mut self, command: JavyCommand) -> &mut Self {
        self.command = command;
        self
    }

    pub fn plugin(&mut self, plugin: Plugin) -> &mut Self {
        self.plugin = plugin;
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
            root,
            javy_stream_io,
            simd_json_builtins,
            text_encoding,
            event_loop,
            built: _,
            preload,
            command,
            plugin,
        } = std::mem::take(self);

        self.built = true;

        match command {
            JavyCommand::Compile => {
                if let Some(preload) = preload {
                    Runner::compile_dynamic(bin_path, root, input, wit, world, preload, plugin)
                } else {
                    Runner::compile_static(bin_path, root, input, wit, world, plugin)
                }
            }
            JavyCommand::Build => Runner::build(
                bin_path,
                root,
                input,
                wit,
                world,
                javy_stream_io,
                simd_json_builtins,
                text_encoding,
                event_loop,
                preload,
                plugin,
            ),
        }
    }
}

pub struct Runner {
    pub wasm: Vec<u8>,
    linker: Linker<StoreContext>,
    initial_fuel: u64,
    preload: Option<(String, Vec<u8>)>,
    plugin: Plugin,
}

#[derive(Debug)]
pub struct RunnerError {
    pub stdout: Vec<u8>,
    pub stderr: String,
    pub err: anyhow::Error,
}

impl Error for RunnerError {}

impl Display for RunnerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error: {:?}, stdout: {:?}, stderr: {}",
            self.err, self.stdout, self.stderr
        )
    }
}

struct StoreContext {
    wasi: Option<WasiCtx>,
    logs: WritePipe<LogWriter>,
    output: WritePipe<Cursor<Vec<u8>>>,
}

impl StoreContext {
    fn new(capacity: usize, input: &[u8]) -> Self {
        let output = WritePipe::new_in_memory();
        let logs = WritePipe::new(LogWriter::new(capacity));
        let wasi = WasiCtxBuilder::new()
            .stdin(Box::new(ReadPipe::from(input)))
            .stdout(Box::new(output.clone()))
            .stderr(Box::new(logs.clone()))
            .build();

        Self {
            wasi: Some(wasi),
            output,
            logs,
        }
    }
}

pub enum UseExportedFn {
    EvalBytecode,
    Invoke(Option<&'static str>),
}

impl Runner {
    #[allow(clippy::too_many_arguments)]
    fn build(
        bin: String,
        root: PathBuf,
        source: impl AsRef<Path>,
        wit: Option<PathBuf>,
        world: Option<String>,
        javy_stream_io: Option<bool>,
        override_json_parse_and_stringify: Option<bool>,
        text_encoding: Option<bool>,
        event_loop: Option<bool>,
        preload: Option<(String, PathBuf)>,
        plugin: Plugin,
    ) -> Result<Self> {
        // This directory is unique and will automatically get deleted
        // when `tempdir` goes out of scope.
        let tempdir = tempfile::tempdir()?;
        let wasm_file = Self::out_wasm(&tempdir);
        let js_file = root.join(source);
        let wit_file = wit.map(|p| root.join(p));

        let args = Self::build_args(
            &js_file,
            &wasm_file,
            &wit_file,
            &world,
            preload.is_some(),
            &javy_stream_io,
            &override_json_parse_and_stringify,
            &text_encoding,
            &event_loop,
            &plugin,
        );

        Self::exec_command(bin, root, args)?;

        let wasm = fs::read(&wasm_file)?;

        let engine = Self::setup_engine();
        let linker = Self::setup_linker(&engine)?;

        let preload = preload
            .map(|(name, path)| {
                let module = fs::read(path)?;
                Ok::<(String, Vec<u8>), anyhow::Error>((name, module))
            })
            .transpose()?;

        Ok(Self {
            wasm,
            linker,
            initial_fuel: u64::MAX,
            preload,
            plugin: Plugin::Default,
        })
    }

    fn compile_static(
        bin: String,
        root: PathBuf,
        source: impl AsRef<Path>,
        wit: Option<PathBuf>,
        world: Option<String>,
        plugin: Plugin,
    ) -> Result<Self> {
        // This directory is unique and will automatically get deleted
        // when `tempdir` goes out of scope.
        let tempdir = tempfile::tempdir()?;
        let wasm_file = Self::out_wasm(&tempdir);
        let js_file = root.join(source);
        let wit_file = wit.map(|p| root.join(p));

        let args = Self::base_compile_args(&js_file, &wasm_file, &wit_file, &world);

        Self::exec_command(bin, root, args)?;

        let wasm = fs::read(&wasm_file)?;

        let engine = Self::setup_engine();
        let linker = Self::setup_linker(&engine)?;

        Ok(Self {
            wasm,
            linker,
            initial_fuel: u64::MAX,
            preload: None,
            plugin,
        })
    }

    pub fn compile_dynamic(
        bin: String,
        root: PathBuf,
        source: impl AsRef<Path>,
        wit: Option<PathBuf>,
        world: Option<String>,
        preload: (String, PathBuf),
        plugin: Plugin,
    ) -> Result<Self> {
        let tempdir = tempfile::tempdir()?;
        let wasm_file = Self::out_wasm(&tempdir);
        let js_file = root.join(source);
        let wit_file = wit.map(|p| root.join(p));

        let mut args = Self::base_compile_args(&js_file, &wasm_file, &wit_file, &world);
        args.push("-d".to_string());

        Self::exec_command(bin, root, args)?;

        let wasm = fs::read(&wasm_file)?;
        let preload_module = fs::read(&preload.1)?;

        let engine = Self::setup_engine();
        let linker = Self::setup_linker(&engine)?;

        Ok(Self {
            wasm,
            linker,
            initial_fuel: u64::MAX,
            preload: Some((preload.0, preload_module)),
            plugin,
        })
    }

    pub fn with_dylib(wasm: Vec<u8>) -> Result<Self> {
        let engine = Self::setup_engine();
        Ok(Self {
            wasm,
            linker: Self::setup_linker(&engine)?,
            initial_fuel: u64::MAX,
            preload: None,
            plugin: Plugin::Default,
        })
    }

    pub fn ensure_expected_imports(&self, expect_eval_bytecode: bool) -> Result<()> {
        let module = Module::from_binary(self.linker.engine(), &self.wasm)?;
        let instance_name = self.plugin.namespace();

        let imports = module
            .imports()
            .filter(|i| i.module() == instance_name)
            .collect::<Vec<_>>();
        let expected_import_count = if expect_eval_bytecode { 4 } else { 3 };
        if imports.len() != expected_import_count {
            bail!("Dynamically linked modules should have exactly {expected_import_count} imports for {instance_name}");
        }

        let realloc = imports
            .iter()
            .find(|i| i.name() == "canonical_abi_realloc")
            .ok_or_else(|| anyhow!("Should have canonical_abi_realloc import"))?;
        let ty = realloc.ty();
        let f = ty.unwrap_func();
        if !f.params().all(|p| p.is_i32()) || f.params().len() != 4 {
            bail!("canonical_abi_realloc should accept 4 i32s as parameters");
        }
        if !f.results().all(|p| p.is_i32()) || f.results().len() != 1 {
            bail!("canonical_abi_realloc should return 1 i32 as a result");
        }

        imports
            .iter()
            .find(|i| i.name() == "memory" && i.ty().memory().is_some())
            .ok_or_else(|| anyhow!("Should have memory import named memory"))?;

        if expect_eval_bytecode {
            let eval_bytecode = imports
                .iter()
                .find(|i| i.name() == "eval_bytecode")
                .ok_or_else(|| anyhow!("Should have eval_bytecode import"))?;
            let ty = eval_bytecode.ty();
            let f = ty.unwrap_func();
            if !f.params().all(|p| p.is_i32()) || f.params().len() != 2 {
                bail!("eval_bytecode should accept 2 i32s as parameters");
            }
            if f.results().len() != 0 {
                bail!("eval_bytecode should return no results");
            }
        }

        let invoke = imports
            .iter()
            .find(|i| i.name() == "invoke")
            .ok_or_else(|| anyhow!("Should have invoke import"))?;
        let ty = invoke.ty();
        let f = ty.unwrap_func();
        if !f.params().all(|p| p.is_i32()) || f.params().len() != 4 {
            bail!("invoke should accept 4 i32s as parameters");
        }
        if f.results().len() != 0 {
            bail!("invoke should return no results");
        }

        Ok(())
    }

    pub fn assert_producers(&self) -> Result<()> {
        let producers_section = wasmparser::Parser::new(0)
            .parse_all(&self.wasm)
            .find_map(|payload| {
                if let Ok(wasmparser::Payload::CustomSection(c)) = payload {
                    if let wasmparser::KnownCustom::Producers(r) = c.as_known() {
                        return Some(r);
                    }
                }
                None
            })
            .expect("Should have producers custom section");
        let fields = producers_section
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(2, fields.len());

        let language_field = &fields[0];
        assert_eq!("language", language_field.name);
        assert_eq!(1, language_field.values.count());
        let language_value = language_field.values.clone().into_iter().next().unwrap()?;
        assert_eq!("JavaScript", language_value.name);
        assert_eq!("ES2020", language_value.version);

        let processed_by_field = &fields[1];
        assert_eq!("processed-by", processed_by_field.name);
        assert_eq!(1, processed_by_field.values.count());
        let processed_by_value = processed_by_field
            .values
            .clone()
            .into_iter()
            .next()
            .unwrap()?;
        assert_eq!("Javy", processed_by_value.name);
        Ok(())
    }

    fn out_wasm(dir: &TempDir) -> PathBuf {
        let name = format!("{}.wasm", uuid::Uuid::new_v4());
        let file = dir.path().join(name);
        file
    }

    // TODO: Some of the methods in the Runner (`build`, `build_args`)  could be
    // refactored to take structs as parameters rather than individual
    // parameters to avoid verbosity.
    //
    // This refactoring will be a bit challenging until we fully deprecate the
    // `compile` command.
    #[allow(clippy::too_many_arguments)]
    fn build_args(
        input: &Path,
        out: &Path,
        wit: &Option<PathBuf>,
        world: &Option<String>,
        dynamic: bool,
        javy_stream_io: &Option<bool>,
        simd_json_builtins: &Option<bool>,
        text_encoding: &Option<bool>,
        event_loop: &Option<bool>,
        plugin: &Plugin,
    ) -> Vec<String> {
        let mut args = vec![
            "build".to_string(),
            input.to_str().unwrap().to_string(),
            "-o".to_string(),
            out.to_str().unwrap().to_string(),
        ];

        if let (Some(wit), Some(world)) = (wit, world) {
            args.push("-C".to_string());
            args.push(format!("wit={}", wit.to_str().unwrap()));
            args.push("-C".to_string());
            args.push(format!("wit-world={world}"));
        }

        if dynamic {
            args.push("-C".to_string());
            args.push("dynamic".to_string());
        }

        if let Some(enabled) = *javy_stream_io {
            args.push("-J".to_string());
            args.push(format!(
                "javy-stream-io={}",
                if enabled { "y" } else { "n" }
            ));
        }

        if let Some(enabled) = *simd_json_builtins {
            args.push("-J".to_string());
            args.push(format!(
                "simd-json-builtins={}",
                if enabled { "y" } else { "n" }
            ));
        }

        if let Some(enabled) = *text_encoding {
            args.push("-J".to_string());
            args.push(format!("text-encoding={}", if enabled { "y" } else { "n" }));
        }

        if let Some(enabled) = *event_loop {
            args.push("-J".to_string());
            args.push(format!("event-loop={}", if enabled { "y" } else { "n" }));
        }

        if matches!(plugin, Plugin::User | Plugin::DefaultAsUser) {
            args.push("-C".to_string());
            args.push(format!("plugin={}", plugin.path().to_str().unwrap()));
        }

        args
    }

    fn base_compile_args(
        input: &Path,
        out: &Path,
        wit: &Option<PathBuf>,
        world: &Option<String>,
    ) -> Vec<String> {
        let mut args = vec![
            "compile".to_string(),
            input.to_str().unwrap().to_string(),
            "-o".to_string(),
            out.to_str().unwrap().to_string(),
        ];

        if let (Some(wit), Some(world)) = (wit, world) {
            args.push("--wit".to_string());
            args.push(wit.to_str().unwrap().to_string());
            args.push("-n".to_string());
            args.push(world.to_string());
        }

        args
    }

    fn exec_command(bin: String, root: PathBuf, args: Vec<String>) -> Result<()> {
        let output = Command::new(bin).current_dir(root).args(args).output()?;

        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;

        if !output.status.success() {
            bail!(
                "terminated with status = {}, stderr = {}",
                output.status,
                std::str::from_utf8(&output.stderr).unwrap(),
            );
        }

        Ok(())
    }

    fn setup_engine() -> Engine {
        let mut config = Config::new();
        config.cranelift_opt_level(OptLevel::SpeedAndSize);
        config.consume_fuel(true);
        Engine::new(&config).expect("failed to create engine")
    }

    fn setup_linker(engine: &Engine) -> Result<Linker<StoreContext>> {
        let mut linker = Linker::new(engine);

        wasi_common::sync::add_to_linker(&mut linker, |ctx: &mut StoreContext| {
            ctx.wasi.as_mut().unwrap()
        })?;

        Ok(linker)
    }

    fn setup_store(engine: &Engine, input: &[u8]) -> Result<Store<StoreContext>> {
        let mut store = Store::new(engine, StoreContext::new(usize::MAX, input));
        store.set_fuel(u64::MAX)?;
        Ok(store)
    }

    pub fn exec(&mut self, input: &[u8]) -> Result<(Vec<u8>, Vec<u8>, u64)> {
        self.exec_func("_start", input)
    }

    pub fn exec_func(&mut self, func: &str, input: &[u8]) -> Result<(Vec<u8>, Vec<u8>, u64)> {
        let mut store = Self::setup_store(self.linker.engine(), input)?;
        let module = Module::from_binary(self.linker.engine(), &self.wasm)?;

        if let Some((name, bytes)) = &self.preload {
            let module = Module::from_binary(self.linker.engine(), bytes)?;
            // Allow unknown imports for dynamically linked `test-plugin`.
            self.linker.define_unknown_imports_as_traps(&module)?;
            let instance = self.linker.instantiate(store.as_context_mut(), &module)?;
            self.linker.allow_shadowing(true);
            self.linker
                .instance(store.as_context_mut(), name, instance)?;
        }

        // Allow unknown imports for statically linked `test-plugin`.
        self.linker.define_unknown_imports_as_traps(&module)?;
        let instance = self.linker.instantiate(store.as_context_mut(), &module)?;
        let run = instance.get_typed_func::<(), ()>(store.as_context_mut(), func)?;

        let res = run.call(store.as_context_mut(), ());

        self.extract_store_data(res, store)
    }

    pub fn exec_through_dylib(
        &mut self,
        src: &str,
        use_exported_fn: UseExportedFn,
    ) -> Result<(Vec<u8>, Vec<u8>, u64)> {
        let mut store = Self::setup_store(self.linker.engine(), &[])?;
        let module = Module::from_binary(self.linker.engine(), &self.wasm)?;

        let instance = self.linker.instantiate(store.as_context_mut(), &module)?;

        let (bc_ptr, bc_len) = Self::compile(src.as_bytes(), store.as_context_mut(), &instance)?;
        let res = match use_exported_fn {
            UseExportedFn::EvalBytecode => instance
                .get_typed_func::<(u32, u32), ()>(store.as_context_mut(), "eval_bytecode")?
                .call(store.as_context_mut(), (bc_ptr, bc_len)),
            UseExportedFn::Invoke(func) => {
                let (fn_ptr, fn_len) = match func {
                    Some(func) => Self::copy_func_name(func, &instance, store.as_context_mut())?,
                    None => (0, 0),
                };
                instance
                    .get_typed_func::<(u32, u32, u32, u32), ()>(store.as_context_mut(), "invoke")?
                    .call(store.as_context_mut(), (bc_ptr, bc_len, fn_ptr, fn_len))
            }
        };

        self.extract_store_data(res, store)
    }

    fn copy_func_name(
        name: &str,
        instance: &Instance,
        mut store: impl AsContextMut,
    ) -> Result<(u32, u32)> {
        let memory = instance
            .get_memory(store.as_context_mut(), "memory")
            .unwrap();
        let fn_name_bytes = name.as_bytes();
        let fn_name_ptr = Self::allocate_memory(
            instance,
            store.as_context_mut(),
            1,
            fn_name_bytes.len().try_into()?,
        )?;
        memory.write(
            store.as_context_mut(),
            fn_name_ptr.try_into()?,
            fn_name_bytes,
        )?;

        Ok((fn_name_ptr, fn_name_bytes.len().try_into()?))
    }

    fn compile(
        source: &[u8],
        mut store: impl AsContextMut,
        instance: &Instance,
    ) -> Result<(u32, u32)> {
        let memory = instance
            .get_memory(store.as_context_mut(), "memory")
            .unwrap();
        let compile_src_func =
            instance.get_typed_func::<(u32, u32), u32>(store.as_context_mut(), "compile_src")?;

        let js_src_ptr = Self::allocate_memory(
            instance,
            store.as_context_mut(),
            1,
            source.len().try_into()?,
        )?;
        memory.write(store.as_context_mut(), js_src_ptr.try_into()?, source)?;

        let ret_ptr = compile_src_func.call(
            store.as_context_mut(),
            (js_src_ptr, source.len().try_into()?),
        )?;
        let mut ret_buffer = [0; 8];
        memory.read(store.as_context(), ret_ptr.try_into()?, &mut ret_buffer)?;
        let bytecode_ptr = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
        let bytecode_len = u32::from_le_bytes(ret_buffer[4..8].try_into()?);

        Ok((bytecode_ptr, bytecode_len))
    }

    fn allocate_memory(
        instance: &Instance,
        mut store: impl AsContextMut,
        alignment: u32,
        new_size: u32,
    ) -> Result<u32> {
        let realloc_func = instance.get_typed_func::<(u32, u32, u32, u32), u32>(
            store.as_context_mut(),
            "canonical_abi_realloc",
        )?;
        let orig_ptr = 0;
        let orig_size = 0;
        realloc_func
            .call(
                store.as_context_mut(),
                (orig_ptr, orig_size, alignment, new_size),
            )
            .map_err(Into::into)
    }

    fn extract_store_data(
        &self,
        call_result: Result<()>,
        mut store: Store<StoreContext>,
    ) -> Result<(Vec<u8>, Vec<u8>, u64)> {
        let fuel_consumed = self.initial_fuel - store.as_context_mut().get_fuel()?;
        let store_context = store.into_data();
        drop(store_context.wasi);
        let logs = store_context
            .logs
            .try_into_inner()
            .expect("log stream reference still exists")
            .buffer;
        let output = store_context
            .output
            .try_into_inner()
            .expect("Output stream reference still exists")
            .into_inner();

        match call_result {
            Ok(_) => Ok((output, logs, fuel_consumed)),
            Err(err) => Err(RunnerError {
                stdout: output,
                stderr: String::from_utf8(logs).unwrap(),
                err,
            }
            .into()),
        }
    }
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
