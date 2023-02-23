use anyhow::{anyhow, bail, Result};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use num_format::{Locale, ToFormattedString};
use std::{fmt::Display, fs, path::Path, process::Command};
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
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl Function {
    pub fn new(function_dir: &Path, js_path: &Path) -> Result<Function> {
        let name = function_dir
            .file_name()
            .ok_or(anyhow!("Path terminates in .."))?
            .to_str()
            .ok_or(anyhow!("Function file name contains invalid unicode"))?
            .to_string();

        let wasm_path = function_dir.join("index.wasm");
        execute_javy(&function_dir.join(js_path), &wasm_path)?;

        Ok(Function {
            name,
            wasm_bytes: fs::read(wasm_path)?,
            payload: fs::read(function_dir.join("input.json"))?,
            engine: Engine::new(&Config::default())?,
        })
    }

    pub fn compile(&self) -> Result<Vec<u8>> {
        let module = Module::new(&self.engine, &self.wasm_bytes)?.serialize()?;
        Ok(module)
    }

    pub fn run_precompiled(
        &self,
        elf_js_module: &[u8],
        linker: &mut Linker<WasiCtx>,
        store: &mut Store<WasiCtx>,
    ) -> Result<()> {
        let js_module = unsafe { Module::deserialize(&self.engine, elf_js_module) }?;
        self.run(&js_module, linker, store)?;
        Ok(())
    }

    pub fn run_uncompiled(
        &self,
        linker: &mut Linker<WasiCtx>,
        store: &mut Store<WasiCtx>,
    ) -> Result<()> {
        let js_module = Module::new(&self.engine, &self.wasm_bytes)?;
        self.run(&js_module, linker, store)?;
        Ok(())
    }

    fn run(
        &self,
        js_module: &Module,
        linker: &mut Linker<WasiCtx>,
        mut store: &mut Store<WasiCtx>,
    ) -> Result<()> {
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

    pub fn setup(&self) -> Result<(Linker<WasiCtx>, Store<WasiCtx>)> {
        let mut linker = Linker::new(&self.engine);
        let wasi = WasiCtxBuilder::new()
            .stdin(Box::new(ReadPipe::from(&self.payload[..])))
            .stdout(Box::new(WritePipe::new_in_memory()))
            .inherit_stderr()
            .build();
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
        let store = Store::new(&self.engine, wasi);
        Ok((linker, store))
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let functions = vec![
        Function::new(
            Path::new("benches/functions/simple_discount"),
            Path::new("index.js"),
        )
        .unwrap(),
        Function::new(
            Path::new("benches/functions/complex_discount"),
            Path::new("dist/function.js"),
        )
        .unwrap(),
    ];

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

fn execute_javy(index_js: &Path, wasm: &Path) -> Result<()> {
    let status_code = Command::new(Path::new("../../target/release/javy").to_str().unwrap())
        .args([
            "compile",
            index_js.to_str().unwrap(),
            "-o",
            wasm.to_str().unwrap(),
        ])
        .status()?;
    if !status_code.success() {
        bail!("Javy exited with non-zero exit code");
    }
    Ok(())
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
