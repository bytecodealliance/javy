//! Javy's profiler entrypoint.

use anyhow::{Result, anyhow};
use javy_profiler_lib::monitor;
use wasmtime::{Engine, Linker, Store};
use wasmtime_wasi::{WasiCtxBuilder, p2::pipe::MemoryInputPipe};
use wasmtime_wizer::Wizer;
use whamm::api::instrument::{UserLibs, instrument_with_rewriting};

/// The profiler state library.
const PROFILER_LIB_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/profiler_lib.wasm"));

/// Re-export of the profiler library's import namespace.
pub use javy_profiler_lib::LIBRARY_NAME;

/// The pair of artifacts produced by [`inject`].
pub struct ProfileOutput {
    /// The original module rewritten with whamm probes.
    pub instrumented: Vec<u8>,
    /// The wizened state library that backs the runtime imports
    /// injected by whamm.
    pub state_lib: Vec<u8>,
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
pub async fn inject(wasm: Vec<u8>) -> Result<ProfileOutput> {
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
