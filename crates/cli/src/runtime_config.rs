use anyhow::{anyhow, Result};
use serde::Serialize;
use wasi_common::{sync::WasiCtxBuilder, WasiCtx};
use wasmtime::{AsContextMut, Engine, Instance, Linker, Module, Store, Val};

use crate::providers::Provider;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Config {
    pub simd_json_builtins: bool,
    pub javy_json: bool,
    pub javy_stream_io: bool,
    pub redirect_stdout_to_stderr: bool,
    pub text_encoding: bool,
    pub extra_field: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            simd_json_builtins: true,
            javy_json: true,
            javy_stream_io: true,
            redirect_stdout_to_stderr: true,
            text_encoding: true,
            extra_field: true,
        }
    }
}

pub fn generate_config_string(provider: Provider, config: Config) -> Result<String> {
    let engine = Engine::default();
    let module = Module::new(&engine, provider.as_bytes())?;
    let mut linker = Linker::new(&engine);
    wasi_common::sync::snapshots::preview_1::add_wasi_snapshot_preview1_to_linker(
        &mut linker,
        |s| s,
    )?;
    let wasi = WasiCtxBuilder::new().inherit_stderr().build();
    let mut store = Store::new(&engine, wasi);
    let instance = linker.instantiate(store.as_context_mut(), &module)?;
    set_property(
        &instance,
        &mut store,
        "config_simd_json_builtins",
        config.simd_json_builtins,
    )?;
    set_property(&instance, &mut store, "config_javy_json", config.javy_json)?;
    set_property(
        &instance,
        &mut store,
        "config_javy_stream_io",
        config.javy_stream_io,
    )?;
    set_property(
        &instance,
        &mut store,
        "config_redirect_stdout_to_stderr",
        config.redirect_stdout_to_stderr,
    )?;
    set_property(&instance, &mut store, "text_encoding", config.text_encoding)?;
    let config = instance
        .get_typed_func::<(), u32>(store.as_context_mut(), "generate_config")?
        .call(store.as_context_mut(), ())?;
    Ok(config.to_string())
}

fn set_property(
    instance: &Instance,
    store: &mut Store<WasiCtx>,
    name: &str,
    prop: bool,
) -> Result<()> {
    instance
        .get_export(store.as_context_mut(), name)
        .ok_or_else(|| anyhow!("Cannot {name}"))?
        .into_global()
        .ok_or_else(|| anyhow!("{name} should be a global"))?
        .set(
            store.as_context_mut(),
            if prop { Val::I32(1) } else { Val::I32(0) },
        )
}
