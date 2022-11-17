use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use num_format::{Locale, ToFormattedString};
use std::{error::Error, fmt::Display, fs, path::Path, process::Command};
use wasi_common::{
    pipe::{ReadPipe, WritePipe},
    WasiCtx,
};
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;

struct Function {
    name: String,
    wasm_bytes: Vec<u8>,
    payload: Vec<u8>,
    engine: Engine,
    quickjs_provider: Module,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl Function {
    pub fn new(function_dir: &Path) -> Result<Function, Box<dyn Error>> {
        let name = function_dir
            .file_name()
            .ok_or("Path terminates in ..")?
            .to_str()
            .ok_or("Function file name contains invalid unicode")?
            .to_string();

        let wasm_path = function_dir.join("index.wasm");
        execute_javy(&function_dir.join("index.js"), &wasm_path);

        let engine = Engine::new(&Config::default().wasm_multi_memory(true))?;
        let quickjs_provider = Module::from_file(
            &engine,
            Path::new("..")
                .join("..")
                .join("javy_core.init_engine_wizened.wasm"),
        )?;

        Ok(Function {
            name,
            wasm_bytes: fs::read(wasm_path)?,
            payload: fs::read(function_dir.join("input.json"))?,
            engine,
            quickjs_provider,
        })
    }

    pub fn compile(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let module = Module::new(&self.engine, &self.wasm_bytes)?.serialize()?;
        Ok(module)
    }

    pub fn run_precompiled(
        &self,
        elf_js_module: &[u8],
        linker: &mut Linker<WasiCtx>,
        store: &mut Store<WasiCtx>,
    ) -> Result<(), Box<dyn Error>> {
        let js_module = unsafe { Module::deserialize(&self.engine, elf_js_module) }?;
        self.run(&js_module, linker, store)?;
        Ok(())
    }

    pub fn run_uncompiled(
        &self,
        linker: &mut Linker<WasiCtx>,
        store: &mut Store<WasiCtx>,
    ) -> Result<(), Box<dyn Error>> {
        let js_module = Module::new(&self.engine, &self.wasm_bytes)?;
        self.run(&js_module, linker, store)?;
        Ok(())
    }

    fn run(
        &self,
        js_module: &Module,
        linker: &mut Linker<WasiCtx>,
        mut store: &mut Store<WasiCtx>,
    ) -> Result<(), Box<dyn Error>> {
        let consumer_instance = linker.instantiate(&mut store, &js_module)?;
        linker.instance(&mut store, "consumer", consumer_instance)?;

        linker
            .get(&mut store, "consumer", Some("_start"))
            .unwrap()
            .into_func()
            .unwrap()
            .typed::<(), (), _>(&mut store)?
            .call(&mut store, ())?;
        Ok(())
    }

    pub fn setup(&self) -> Result<(Linker<WasiCtx>, Store<WasiCtx>), Box<dyn Error>> {
        let mut linker = Linker::new(&self.engine);
        let stdout = WritePipe::new_in_memory();
        let wasi = WasiCtxBuilder::new()
            .stdin(Box::new(ReadPipe::from(&self.payload[..])))
            .stdout(Box::new(stdout.clone()))
            .stderr(Box::new(stdout))
            .build();
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
        let mut store = Store::new(&self.engine, wasi);
        let quickjs_provider_instance = linker.instantiate(&mut store, &self.quickjs_provider)?;
        linker.instance(
            &mut store,
            "shopify_std_runtime_js_v1",
            quickjs_provider_instance,
        )?;
        Ok((linker, store))
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let functions = fs::read_dir(Path::new("benches").join("functions"))
        .unwrap()
        .map(|entry| Function::new(&entry.unwrap().path()).unwrap());

    for function in functions {
        c.bench_with_input(
            BenchmarkId::new("uncompiled", &function),
            &function,
            |b, f| {
                b.iter_with_setup(
                    || function.setup().unwrap(),
                    |(mut linker, mut store)| f.run_uncompiled(&mut linker, &mut store).unwrap(),
                )
            },
        );

        let serialized_module = function.compile().unwrap();
        println!(
            "Size of precompiled module for {}: {} bytes",
            function,
            serialized_module.len().to_formatted_string(&Locale::en)
        );

        c.bench_with_input(
            BenchmarkId::new("precompiled", &function),
            &function,
            |b, f| {
                b.iter_with_setup(
                    || function.setup().unwrap(),
                    |(mut linker, mut store)| {
                        f.run_precompiled(&serialized_module, &mut linker, &mut store)
                            .unwrap()
                    },
                )
            },
        );
    }
}

fn execute_javy(index_js: &Path, wasm: &Path) {
    let status_code = Command::new(
        Path::new("..")
            .join("..")
            .join("target")
            .join("release")
            .join("javy")
            .to_str()
            .unwrap(),
    )
    .args([index_js.to_str().unwrap(), "-o", wasm.to_str().unwrap()])
    .status()
    .unwrap();
    if !status_code.success() {
        panic!("Javy exited with non-zero exit code");
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
