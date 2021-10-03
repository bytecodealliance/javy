use std::env;
use std::path::PathBuf;

fn main() {
    let host_platform = match std::env::consts::OS {
        v @ "linux" => v,
        v @ "macos" => v,
        not_supported => panic!("{} is not supported.", not_supported),
    };
    let sysroot = format!(
        "--sysroot=vendor/{}/wasi-sdk/share/wasi-sysroot",
        host_platform
    );

    // Use a custom version of clang/ar with WASI support.
    // They are both vendored within the WASI sdk for both OSX and linux.
    let clang = format!("vendor/{}/wasi-sdk/bin/clang", host_platform);
    env::set_var("CC_wasm32_wasi", &clang);
    env::set_var("CC", &clang);

    let ar = format!("vendor/{}/wasi-sdk/bin/ar", host_platform);
    env::set_var("AR_wasm32_wasi", &ar);
    env::set_var("AR", &ar);

    // Tell clang we need to use the wasi-sysroot instead of the host platform.
    env::set_var("CFLAGS", &sysroot);

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
        .opt_level(3)
        .compile("quickjs");

    // Generate bindings for quickjs
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(&["-fvisibility=default", "--target=wasm32-wasi", &sysroot])
        .generate()
        .unwrap();

    println!("cargo:rerun-if-changed=src/extensions/value.c");
    println!("cargo:rerun-if-changed=wrapper.h");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir.join("bindings.rs")).unwrap();
}
