use std::path::PathBuf;
use std::env;

fn main() {
    let profile = env::var("PROFILE").expect("Couldn't retrieve profile");
    let out_dir: PathBuf = env::var("OUT_DIR").expect("Couldn't retrieve out dir").into();
    let root: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok()
        .map(|mut b| {
            b.pop();
            b.pop();
            b
                .join("target")
                .join("wasm32-wasi")
                .join(profile)
                .join("javy_core.wasm")
        })
        .expect("Couldn't retrieve root dir");

        std::fs::copy(root, out_dir.join("engine.wasm")).unwrap();
}
