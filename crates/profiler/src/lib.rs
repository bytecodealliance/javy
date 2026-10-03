//! Javy's profiler entrypoint.

use anyhow::{anyhow, Result};
use javy_profiler_lib::monitor;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{p2::pipe::MemoryInputPipe, I32Exit, WasiCtxBuilder};
use wasmtime_wizer::Wizer;
use whamm::api::instrument::{instrument_with_rewriting, UserLibs};

/// The profiler state library.
const PROFILER_LIB_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/profiler_lib.wasm"));

/// Re-export of the profiler library's report encoding.
pub use javy_profiler_lib::format;
use javy_profiler_lib::LIBRARY_NAME;

/// Export invoked by default: a WASI command module's entry function.
pub const DEFAULT_INVOKE: &str = "_start";

/// The pair of artifacts produced by [`inject`].
struct ProfileOutput {
    /// The original module rewritten with whamm probes.
    instrumented: Vec<u8>,
    /// The wizened state library that backs the runtime imports
    /// injected by whamm.
    state_lib: Vec<u8>,
}

/// Instrument `wasm`, run it by calling the export `invoke`, and return
/// the serialized report.
///
/// The app's WASI imports are backed by the host's stdio, so the program
/// reads its real input. Any other import is mocked with default values,
/// so programs relying on custom host functions may take different paths
/// than under their real embedder.
pub async fn profile(wasm: Vec<u8>, invoke: &str) -> Result<Vec<u8>> {
    let output = inject(wasm).await?;
    run(&output, invoke).await
}

/// Pre-initialize the profiler state library through wizer.
async fn preinitialize_state_lib(state_lib: &[u8], app_wasm: &[u8]) -> Result<Vec<u8>> {
    let engine = Engine::default();
    let mut builder = WasiCtxBuilder::new();
    builder
        .stdin(MemoryInputPipe::new(app_wasm.to_vec()))
        .inherit_stderr();
    let wasi = builder.build_p1();
    let mut store = Store::new(&engine, wasi);

    Ok(Wizer::new()
        .init_func("wizer.initialize")
        .run(&mut store, state_lib, async |store, module| {
            let engine = store.engine();
            let mut linker = Linker::new(engine);
            wasmtime_wasi::p1::add_to_linker_async(&mut linker, |cx| cx)?;
            linker.define_unknown_imports_as_traps(module)?;
            let instance = linker.instantiate_async(store, module).await?;
            Ok(instance)
        })
        .await?)
}

/// Rewrite the given WebAssembly module, injecting probes as
/// WebAssembly instructions as directed by the given monitor script.
async fn inject(wasm: Vec<u8>) -> Result<ProfileOutput> {
    let initialized_state_lib = preinitialize_state_lib(PROFILER_LIB_MODULE, &wasm).await?;

    let mut user_libs = UserLibs::new();
    user_libs.insert(
        LIBRARY_NAME.to_string(),
        (None, initialized_state_lib.clone()),
    );

    let instrumented = instrument_with_rewriting(wasm, monitor(), user_libs, None, None)
        .map_err(|mut e| {
            e.report();
            anyhow!("Instrumentation failed. This is considered a bug. Please report this behavior upstream.")
        })?;

    Ok(ProfileOutput {
        instrumented,
        state_lib: initialized_state_lib,
    })
}

/// Run an instrumented module to completion by calling `invoke`, then
/// have its state library serialize the report and copy it out.
async fn run(output: &ProfileOutput, invoke: &str) -> Result<Vec<u8>> {
    let engine = Engine::default();
    let wasi = WasiCtxBuilder::new().inherit_stdio().build_p1();
    let mut store = Store::new(&engine, wasi);

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p1::add_to_linker_async(&mut linker, |cx| cx)?;

    let state_lib = Module::new(&engine, &output.state_lib)?;
    linker.define_unknown_imports_as_default_values(&mut store, &state_lib)?;
    let state_lib = linker.instantiate_async(&mut store, &state_lib).await?;
    linker.instance(&mut store, LIBRARY_NAME, state_lib)?;

    let app = Module::new(&engine, &output.instrumented)?;
    linker.define_unknown_imports_as_default_values(&mut store, &app)?;
    let app = linker.instantiate_async(&mut store, &app).await?;
    let func = app.get_typed_func::<(), ()>(&mut store, invoke)?;
    if let Err(e) = func.call_async(&mut store, ()).await {
        if e.downcast_ref::<I32Exit>().is_none() {
            return Err(e.into());
        }
    }

    let report = state_lib.get_typed_func::<(), ()>(&mut store, "report")?;
    let report_ptr = state_lib.get_typed_func::<(), u32>(&mut store, "report_ptr")?;
    let report_len = state_lib.get_typed_func::<(), u32>(&mut store, "report_len")?;
    report.call_async(&mut store, ()).await?;
    let ptr = report_ptr.call_async(&mut store, ()).await? as usize;
    let len = report_len.call_async(&mut store, ()).await? as usize;

    let memory = state_lib
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow!("the state library does not export its memory"))?;
    let bytes = memory
        .data(&store)
        .get(ptr..ptr + len)
        .ok_or_else(|| anyhow!("report buffer {ptr:#x}+{len} is out of bounds"))?;
    Ok(bytes.to_vec())
}
