use anyhow::Result;
use std::{fs, rc::Rc};
use walrus::{CustomSection, FunctionId, ImportKind};
use wasmparser::{Parser, Payload};
use wasmtime::Module;
use wasmtime_wasi::WasiCtxBuilder;
use wizer::{Linker, Wizer};

pub fn extract_core_module(component_bytes: &[u8]) -> Result<Vec<u8>> {
    let parser = Parser::new(0);

    for payload in parser.parse_all(component_bytes) {
        match payload? {
            Payload::ModuleSection {
                parser,
                unchecked_range,
                ..
            } => {
                let module_bytes = &component_bytes[unchecked_range.start..unchecked_range.end];
                let mut import_namespace = None;
                for payload in parser.parse_all(module_bytes) {
                    match payload? {
                        Payload::ExportSection(exports) => {
                            for export in exports {
                                let export = export?;
                                import_namespace = export.name.strip_suffix("#invoke");
                                if import_namespace.is_some() {
                                    break;
                                }
                            }
                        }
                        _ => continue,
                    }
                }
                if let Some(import_namespace) = import_namespace {
                    let module_bytes = strip_wasi_p2_imports(module_bytes)?;
                    let module_bytes =
                        add_import_namespace(&module_bytes, import_namespace.to_string())?;
                    return Ok(module_bytes);
                }
            }
            _ => {}
        }
    }

    anyhow::bail!("No suitable module found in component");
}

fn strip_wasi_p2_imports(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut module = walrus::Module::from_buffer(wasm_bytes)?;
    let wasi_p2_imports = module
        .imports
        .iter()
        .filter_map(|import| match import.kind {
            ImportKind::Function(id)
                if import.module.starts_with("wasi:") || import.name == "adapter_close_badfd" =>
            {
                Some(id)
            }
            _ => None,
        })
        .collect::<Vec<FunctionId>>();

    for import in wasi_p2_imports {
        module.replace_imported_func(import, |(builder, _)| {
            builder.func_body().unreachable();
        })?;
    }
    Ok(module.emit_wasm())
}

#[derive(Debug)]
struct ImportNamespaceCustomSection {
    namespace: String,
}

impl ImportNamespaceCustomSection {
    fn new(namespace: String) -> Self {
        Self { namespace }
    }
}

impl CustomSection for ImportNamespaceCustomSection {
    fn name(&self) -> &str {
        "import_namespace"
    }

    fn data(&self, _ids_to_indices: &walrus::IdsToIndices) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Borrowed(self.namespace.as_bytes())
    }
}

fn add_import_namespace(wasm_bytes: &[u8], import_namespace: String) -> Result<Vec<u8>> {
    let mut module = walrus::Module::from_buffer(wasm_bytes)?;
    module
        .customs
        .add(ImportNamespaceCustomSection::new(import_namespace));
    Ok(module.emit_wasm())
}

pub fn optimize_module(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let temp_dir = tempfile::tempdir()?;
    let infile = temp_dir.path().join("infile.wasm");
    fs::write(&infile, wasm_bytes)?;
    let outfile = temp_dir.path().join("outfile.wasm");
    wasm_opt::OptimizationOptions::new_opt_level_4().run(&infile, &outfile)?;
    let optimized_wasm_bytes = fs::read(outfile)?;
    Ok(optimized_wasm_bytes)
}

pub fn preinitialize_module(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut wizer = Wizer::new();
    let owned_wasm_bytes = wasm_bytes.to_vec();
    wizer
        .init_func("bytecodealliance:javy-plugin/javy-plugin-exports@1.0.0#initialize-runtime")
        .keep_init_func(true)
        .make_linker(Some(Rc::new(move |engine| {
            let mut linker = Linker::new(engine);
            wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |ctx| {
                if ctx.wasi_ctx.is_none() {
                    ctx.wasi_ctx = Some(WasiCtxBuilder::new().inherit_stderr().build_p1());
                }
                ctx.wasi_ctx.as_mut().unwrap()
            })?;
            linker.define_unknown_imports_as_traps(&Module::new(engine, &owned_wasm_bytes)?)?;
            Ok(linker)
        })))?
        .run(wasm_bytes)
}
