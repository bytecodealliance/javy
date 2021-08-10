use std::env;
use std::path::PathBuf;

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
    let out_dir: PathBuf = env::var("OUT_DIR")
        .expect("failed to retrieve out dir")
        .into();

    let engine_path = out_dir.join("engine.wasm");
    if !engine_path.exists() {
        std::fs::write(engine_path, "").expect("failed to write empty engine.wasm stub");
        println!("cargo:warning=using stubbed engine.wasm for static analysis purposes...");
    }
}

fn copy_engine_binary() {
    let profile = env::var("PROFILE").expect("Couldn't retrieve profile");
    if profile != "release" {
        eprintln!("only --release is supported due to https://github.com/bytecodealliance/wizer/issues/27");
        std::process::exit(1);
    }

    let out_dir: PathBuf = env::var("OUT_DIR")
        .expect("failed to retrieve out dir")
        .into();

    let engine_path: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok()
        .map(|mut b| {
            b.pop();
            b.pop();
            b.join("target")
                .join("wasm32-wasi")
                .join(profile)
                .join("javy_core.wasm")
        })
        .expect("failed to create path");

    println!("cargo:rerun-if-changed={:?}", engine_path);

    // Only copy the file when it exists. Cargo will take care of re-running this script when the file changes.
    if engine_path.exists() {
        std::fs::copy(&engine_path, out_dir.join("engine.wasm"))
            .unwrap_or_else(|_| panic!("failed to copy engine from {:?}", engine_path));
    }
}

// Copy OS specific prebuild binaries to a known location. The binaries will be embedded in the final binary and
// extracted to a temporary location if it's not already installed.
fn copy_prebuilt_binaries() {
    let target_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let target_os = PathBuf::from(std::env::var("CARGO_CFG_TARGET_OS").unwrap());
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

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
