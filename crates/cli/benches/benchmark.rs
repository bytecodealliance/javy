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
    linking: Linking,
}

impl Display for FunctionCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            match self.linking {
                Linking::Dynamic => "dynamic",
                Linking::Static => "static",
            },
            if self.precompiled_elf_bytes.is_some() {
                "ahead of time"
            } else {
                "just in time"
            },
            self.name
        )
    }
}

impl FunctionCase {
    fn new(
        function_dir: &Path,
        js_path: &Path,
        compilation: &Compilation,
        linking: Linking,
    ) -> Result<Self> {
        let name = function_dir
            .file_name()
            .ok_or_else(|| anyhow!("Path terminates in .."))?
            .to_str()
            .ok_or_else(|| anyhow!("Function file name contains invalid unicode"))?
            .to_string();

        let wasm_path = function_dir.join("index.wasm");
        execute_javy(&function_dir.join(js_path), &wasm_path, &linking)?;

        let engine = Engine::default();
        let wasm_bytes = fs::read(wasm_path)?;

        let precompiled_elf_bytes = match compilation {
            Compilation::AheadOfTime => Some(Module::new(&engine, &wasm_bytes)?.serialize()?),
            Compilation::JustInTime => None,
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
            linking,
        };

        println!(
            "Size of module for {}: {} bytes",
            function_case,
            module_size.to_formatted_string(&Locale::en),
        );

        Ok(function_case)
    }

    #[allow(clippy::needless_borrows_for_generic_args)]
    pub fn run(&self, linker: &mut Linker<WasiCtx>, mut store: &mut Store<WasiCtx>) -> Result<()> {
        let js_module = match &self.precompiled_elf_bytes {
            Some(bytes) => unsafe { Module::deserialize(&self.engine, bytes) }?,
            None => Module::new(&self.engine, &self.wasm_bytes)?,
        };

        let consumer_instance = linker.instantiate(&mut store, &js_module)?;
        linker.instance(&mut store, "consumer", consumer_instance)?;

        linker
            .get(&mut store, "consumer", "_start")
            .unwrap()
            .into_func()
            .unwrap()
            .typed::<(), ()>(&mut store)?
            .call(&mut store, ())?;
        Ok(())
    }

    pub fn setup(&self) -> Result<(Linker<WasiCtx>, Store<WasiCtx>)> {
        let mut linker = Linker::new(&self.engine);
        let wasi = WasiCtxBuilder::new()
            .stdin(Box::new(ReadPipe::from(&self.payload[..])))
            .stdout(Box::new(WritePipe::new_in_memory()))
            .stderr(Box::new(WritePipe::new_in_memory()))
            .build();
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
        let mut store = Store::new(&self.engine, wasi);

        if let Linking::Dynamic = self.linking {
            let qjs_provider = Module::new(
                &self.engine,
                fs::read(Path::new(
                    "../../target/wasm32-wasi/release/javy_quickjs_provider_wizened.wasm",
                ))?,
            )?;
            let instance = linker.instantiate(&mut store, &qjs_provider)?;
            linker.instance(&mut store, "javy_quickjs_provider_v1", instance)?;
        }

        Ok((linker, store))
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut function_cases = vec![];
    for linking in [Linking::Static, Linking::Dynamic] {
        for compilation in [Compilation::JustInTime, Compilation::AheadOfTime] {
            function_cases.push(
                FunctionCase::new(
                    Path::new("benches/functions/simple_discount"),
                    Path::new("index.js"),
                    &compilation,
                    linking,
                )
                .unwrap(),
            );
            function_cases.push(
                FunctionCase::new(
                    Path::new("benches/functions/complex_discount"),
                    Path::new("dist/function.js"),
                    &compilation,
                    linking,
                )
                .unwrap(),
            );
            function_cases.push(
                FunctionCase::new(
                    Path::new("benches/functions/logging"),
                    Path::new("index.js"),
                    &compilation,
                    linking,
                )
                .unwrap(),
            );
        }
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

fn execute_javy(index_js: &Path, wasm: &Path, linking: &Linking) -> Result<()> {
    let mut args = vec![
        "compile",
        index_js.to_str().unwrap(),
        "-o",
        wasm.to_str().unwrap(),
    ];
    if let Linking::Dynamic = linking {
        args.push("-d");
    }
    let status_code = Command::new(Path::new("../../target/release/javy").to_str().unwrap())
        .args(args)
        .status()?;
    if !status_code.success() {
        bail!("Javy exited with non-zero exit code");
    }
    Ok(())
}

enum Compilation {
    AheadOfTime,
    JustInTime,
}

#[derive(Clone, Copy)]
enum Linking {
    Static,
    Dynamic,
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
