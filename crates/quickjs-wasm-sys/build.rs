use std::env;
use std::path::{PathBuf, Path};

use walkdir::WalkDir;

fn main() {
    let this_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let wasi_sdk =
        env::var("QUICKJS_WASM_SYS_WASI_SDK_PATH").unwrap_or(format!("{this_dir}/wasi-sdk"));
    let wasi_sdk_path = Path::new(&wasi_sdk);
    if !Path::exists(wasi_sdk_path) {
        panic!(
            "wasi-sdk not installed in specified path of {}",
            &wasi_sdk
        );
    }
    env::set_var("CC", format!("{}/bin/clang", &wasi_sdk));
    env::set_var("AR", format!("{}/bin/ar", &wasi_sdk));
    let sysroot_path = wasi_sdk_path.join("share").join("wasi-sysroot");
    let sysroot = format!("--sysroot={}", &sysroot_path.display());
    env::set_var("CFLAGS", &sysroot);

    let libclang_path = sysroot_path.join("lib").join("wasm32-wasi");

    println!("cargo:rustc-link-search=native={}", libclang_path.display());
    println!("cargo:rustc-link-lib=static=c");
    println!("cargo:rustc-link-lib=static=c++");


    // Build quickjs as a static library.
    cc::Build::new()
        .files(&[
            "quickjs/cutils.c",
            "quickjs/libbf.c",
            "quickjs/libregexp.c",
            "quickjs/libunicode.c",
            "quickjs/quickjs.c",
            "extensions/value.c",
        ])
        .define("_GNU_SOURCE", None)
        .define("CONFIG_VERSION", "\"2021-03-27\"")
        .define("CONFIG_BIGNUM", None)
        .cargo_metadata(true)
        // The below flags are used by the official Makefile.
        .flag_if_supported("-Wchar-subscripts")
        .flag_if_supported("-Wno-array-bounds")
        .flag_if_supported("-Wno-format-truncation")
        .flag_if_supported("-Wno-missing-field-initializers")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wundef")
        .flag_if_supported("-Wuninitialized")
        .flag_if_supported("-Wunused")
        .flag_if_supported("-Wwrite-strings")
        .flag_if_supported("-funsigned-char")
        .flag_if_supported("-Wno-cast-function-type")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-enum-conversion")
        .flag_if_supported("-Wno-implicit-function-declaration")
        .flag_if_supported("-Wno-implicit-const-int-float-conversion")
        .target("wasm32-wasi")
        .opt_level(2)
        .compile("quickjs");

    // Generate bindings for quickjs
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(&["-fvisibility=default", "--target=wasm32-wasi", &sysroot])
        .generate()
        .unwrap();

    println!("cargo:rerun-if-changed=extensions/value.c");
    println!("cargo:rerun-if-changed=wrapper.h");

    for entry in WalkDir::new("quickjs") {
        println!(
            "cargo:rerun-if-changed={}",
            entry.unwrap().path().to_str().unwrap()
        );
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir.join("bindings.rs")).unwrap();
}
