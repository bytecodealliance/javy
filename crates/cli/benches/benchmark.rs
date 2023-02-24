use anyhow::{anyhow, bail, Result};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use num_format::{Locale, ToFormattedString};
use std::{fmt::Display, fs, path::Path, process::Command};
use wasi_common::{
    pipe::{ReadPipe, WritePipe},
    WasiCtx,
};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;

struct FunctionCase {
    name: String,
    wasm_bytes: Vec<u8>,
    payload: Vec<u8>,
    engine: Engine,
    precompiled_elf_bytes: Option<Vec<u8>>,
}

impl Display for FunctionCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            if self.precompiled_elf_bytes.is_some() {
                "precompiled"
            } else {
                "uncompiled"
            },
            self.name
        )
    }
}

impl FunctionCase {
    pub fn new(function_dir: &Path, js_path: &Path, precompiled: bool) -> Result<Self> {
        let name = function_dir
            .file_name()
            .ok_or(anyhow!("Path terminates in .."))?
            .to_str()
            .ok_or(anyhow!("Function file name contains invalid unicode"))?
            .to_string();

        let wasm_path = function_dir.join("index.wasm");
        execute_javy(&function_dir.join(js_path), &wasm_path)?;

        let engine = Engine::default();
        let wasm_bytes = fs::read(wasm_path)?;

        let precompiled_elf_bytes = if precompiled {
            Some(Module::new(&engine, &wasm_bytes)?.serialize()?)
        } else {
            None
        };
        let module_size = precompiled_elf_bytes
            .as_ref()
            .map(|bs| bs.len())
            .unwrap_or_else(|| wasm_bytes.len());

        let function_case = FunctionCase {
            name,
            wasm_bytes,
            payload: fs::read(function_dir.join("input.json"))?,
            engine,
            precompiled_elf_bytes,
        };

        println!(
            "Size of module for {}: {} bytes",
            function_case,
            module_size.to_formatted_string(&Locale::en),
        );

        Ok(function_case)
    }

    pub fn run(&self, linker: &mut Linker<WasiCtx>, mut store: &mut Store<WasiCtx>) -> Result<()> {
        let js_module = match &self.precompiled_elf_bytes {
            Some(bytes) => unsafe { Module::deserialize(&self.engine, bytes) }?,
            None => Module::new(&self.engine, &self.wasm_bytes)?,
        };

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
    let mut function_cases = vec![];
    for precompiled in [false, true] {
        function_cases.push(
            FunctionCase::new(
                Path::new("benches/functions/simple_discount"),
                Path::new("index.js"),
                precompiled,
            )
            .unwrap(),
        );
        function_cases.push(
            FunctionCase::new(
                Path::new("benches/functions/complex_discount"),
                Path::new("dist/function.js"),
                precompiled,
            )
            .unwrap(),
        );
    }

    for function_case in function_cases {
        c.bench_with_input(
            BenchmarkId::new("run", &function_case),
            &function_case,
            |b, f| {
                b.iter_with_setup(
                    || function_case.setup().unwrap(),
                    |(mut linker, mut store)| f.run(&mut linker, &mut store).unwrap(),
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
