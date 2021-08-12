use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    copy_prebuilt_binaries();

    if let Ok("cargo-clippy") = env::var("CARGO_CFG_FEATURE").as_ref().map(String::as_str) {
        stub_engine_for_clippy();
    } else {
        copy_engine_binary();
    }
}

// When using clippy, we need to write a stubbed engine.wasm file to ensure compilation succeeds. This
// skips building the actual engine.wasm binary that would be injected into the CLI binary.
fn stub_engine_for_clippy() {
    let engine_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("engine.wasm");

    if !engine_path.exists() {
        std::fs::write(engine_path, &[]).expect("failed to write empty engine.wasm stub");
        println!("cargo:warning=using stubbed engine.wasm for static analysis purposes...");
    }
}

// Copy OS specific pre-built binaries to a known location. The binaries will be embedded in the final binary and
// extracted to a temporary location if it's not already installed.
fn copy_prebuilt_binaries() {
    let target_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_os = PathBuf::from(env::var("CARGO_CFG_TARGET_OS").unwrap());
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let vendor_dir = root.join("vendor").join(target_os);
    let target_vendor_dir = target_dir.join("vendor");

    std::fs::create_dir_all(&target_vendor_dir).unwrap();

    std::fs::read_dir(vendor_dir)
        .unwrap()
        .filter_map(Result::ok)
        .for_each(|f| {
            if f.path().is_file() {
                std::fs::copy(f.path(), target_vendor_dir.join(f.file_name())).unwrap();
            }
        });
}

// Copy the engine binary build from the `core` crate, and run wasm-strip + wasm-opt against it as suggested by https://github.com/bytecodealliance/wizer/issues/27.
fn copy_engine_binary() {
    let mut engine_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    engine_path.pop();
    engine_path.pop();
    let engine_path = engine_path.join("target/wasm32-wasi/debug/javy_core.wasm");

    println!("cargo:rerun-if-changed={:?}", engine_path);

    if engine_path.exists() {
        let copied_engine_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("engine.wasm");

        fs::copy(&engine_path, &copied_engine_path).unwrap();
        optimize_engine(&copied_engine_path);
    }
}

fn optimize_engine(engine_path: impl AsRef<Path>) {
    if env::var("JAVY_SKIP_ENGINE_OPTIMIZATIONS").is_ok() {
        return;
    }

    run_wasm_strip(&engine_path);
    run_wasm_opt(&engine_path);
}

fn run_wasm_strip(engine_path: impl AsRef<Path>) {
    let wasm_strip = which::which("wasm-strip")
        .unwrap_or_else(|_| PathBuf::from(env::var("OUT_DIR").unwrap()).join("vendor/wasm-opt"));

    let output = Command::new(wasm_strip)
        .arg(engine_path.as_ref())
        .output()
        .unwrap();

    println!("wasm-strip status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
}

fn run_wasm_opt(engine_path: impl AsRef<Path>) {
    let wasm_opt = which::which("wasm-opt")
        .unwrap_or_else(|_| PathBuf::from(env::var("OUT_DIR").unwrap()).join("vendor/wasm-opt"));

    let output = Command::new(wasm_opt)
        .arg(engine_path.as_ref())
        .arg("-O3")
        .arg("--dce")
        .arg("-o")
        .arg(engine_path.as_ref())
        .output()
        .unwrap();

    println!("wasm-opt status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
}
