use std::env;
use std::path::PathBuf;

fn main() {
    let profile = env::var("PROFILE").expect("Couldn't retrieve profile");
    if profile != "release" {
        eprintln!("only --release is supported due to https://github.com/bytecodealliance/wizer/issues/27");
        std::process::exit(1);
    }

    let out_dir: PathBuf = env::var("OUT_DIR")
        .expect("Couldn't retrieve out dir")
        .into();
    let root: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
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

    if !root.exists() {
        eprintln!("compile core using `cd crates/core && cargo build && cd -`");
        std::process::exit(1);
    }

    println!("cargo:rerun-if-changed={:?}", root);
    std::fs::copy(root, out_dir.join("engine.wasm")).unwrap();
}
