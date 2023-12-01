use anyhow::{anyhow, bail, Error, Result};
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::Response;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use walkdir::WalkDir;

const WASI_SDK_VERSION_MAJOR: usize = 20;
const WASI_SDK_VERSION_MINOR: usize = 0;

async fn download_wasi_sdk() -> Result<PathBuf> {
    let mut wasi_sdk_dir: PathBuf = env::var("OUT_DIR")?.into();
    wasi_sdk_dir.push("wasi-sdk");

    fs::create_dir_all(&wasi_sdk_dir)?;

    const MAJOR_VERSION_ENV_VAR: &str = "QUICKJS_WASM_SYS_WASI_SDK_MAJOR_VERSION";
    const MINOR_VERSION_ENV_VAR: &str = "QUICKJS_WASM_SYS_WASI_SDK_MINOR_VERSION";
    println!("cargo:rerun-if-env-changed={MAJOR_VERSION_ENV_VAR}");
    println!("cargo:rerun-if-env-changed={MINOR_VERSION_ENV_VAR}");
    let major_version =
        env::var(MAJOR_VERSION_ENV_VAR).unwrap_or(WASI_SDK_VERSION_MAJOR.to_string());
    let minor_version =
        env::var(MINOR_VERSION_ENV_VAR).unwrap_or(WASI_SDK_VERSION_MINOR.to_string());

    let mut archive_path = wasi_sdk_dir.clone();
    archive_path.push(format!("wasi-sdk-{major_version}-{minor_version}.tar.gz"));

    // Download archive if necessary
    if !archive_path.try_exists()? {
        let file_suffix = match (env::consts::OS, env::consts::ARCH) {
            ("linux", "x86") | ("linux", "x86_64") => "linux",
            ("macos", "x86") | ("macos", "x86_64") | ("macos", "aarch64") => "macos",
            ("windows", "x86") => "mingw-x86",
            ("windows", "x86_64") => "mingw",
            other => return Err(anyhow!("Unsupported platform tuple {:?}", other)),
        };

        let mut uri = format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{major_version}/wasi-sdk-{major_version}.{minor_version}-{file_suffix}.tar.gz");

        let client = Client::builder(TokioExecutor::new())
            .build::<_, BoxBody<Bytes, Error>>(HttpsConnector::new());
        let mut response: Response<Incoming> = loop {
            let response = client.get(uri.try_into()?).await?;
            let status = response.status();
            if status.is_redirection() {
                uri = response
                    .headers()
                    .get("Location")
                    .ok_or_else(|| anyhow!("Received redirect without location header"))?
                    .to_str()?
                    .to_string();
            } else if !status.is_success() {
                bail!("Received {status} when downloading WASI SDK");
            } else {
                break response;
            }
        };

        let mut archive = fs::File::create(&archive_path)?;

        while let Some(next) = response.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                archive.write_all(chunk)?;
            }
        }
    }

    let mut test_binary = wasi_sdk_dir.clone();
    test_binary.extend(["bin", "wasm-ld"]);
    // Extract archive if necessary
    if !test_binary.try_exists()? {
        let output = process::Command::new("tar")
            .args([
                "-xf",
                archive_path.to_string_lossy().as_ref(),
                "--strip-components",
                "1",
            ])
            .current_dir(&wasi_sdk_dir)
            .output()?;
        if !output.status.success() {
            return Err(anyhow!(
                "Unpacking WASI SDK failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    Ok(wasi_sdk_dir)
}

async fn get_wasi_sdk_path() -> Result<PathBuf> {
    const WASI_SDK_PATH_ENV_VAR: &str = "QUICKJS_WASM_SYS_WASI_SDK_PATH";
    println!("cargo:rerun-if-env-changed={WASI_SDK_PATH_ENV_VAR}");
    if let Ok(path) = env::var(WASI_SDK_PATH_ENV_VAR) {
        return Ok(path.into());
    }
    download_wasi_sdk().await
}

fn find_system_llvm() -> Result<PathBuf> {
    fs::read_dir("/usr/lib")?.find_map(|e| {
        e.as_ref().map_or(None, |e| {
            if e.file_name().to_string_lossy().starts_with("llvm-") {
                Some(e.path())
            } else {
                None
            }
        })
    }).map_or_else(|| Err(anyhow!("Could not determine system llvm version. Is there an llvm installation in /usr/lib?")), Ok)
}

fn copy_system_llvm_to_out_dir() -> Result<PathBuf> {
    let system_llvm_path = find_system_llvm()?;

    let new_llvm_path = PathBuf::from(&format!("{}/llvm", env::var("OUT_DIR")?));
    if new_llvm_path.exists() {
        fs::remove_dir_all(&new_llvm_path)?;
    }

    for file in WalkDir::new(&system_llvm_path) {
        let file = file?;
        let path = file.path();
        let dest_path = new_llvm_path.join(path.strip_prefix(&system_llvm_path)?);
        if path.is_dir() {
            fs::create_dir(&dest_path)?;
            continue;
        }
        if path.is_symlink() {
            continue;
        }
        fs::copy(path, dest_path)?;
    }

    Ok(new_llvm_path)
}

fn install_vendored_libclang_rt_builtins(llvm_path: &Path) -> Result<()> {
    let exit_code = process::Command::new("tar")
        .args([
            "-xf",
            &format!(
                "{}/vendored/libclang_rt.builtins-wasm32-wasi-20.0.tar.gz",
                env!("CARGO_MANIFEST_DIR")
            ),
        ])
        .current_dir(llvm_path)
        .status()?;
    if !exit_code.success() {
        bail!("Failed to extract libclang_rt.builtins-wasm32-wasi archive");
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (clang_path, ar_path, sysroot) = if env::var("DOCS_RS").is_ok() {
        // docs.rs enforces two restrictions that are relevant here:
        // 1. We don't have network access
        // 2. We can't modify anything on the filesystem outside of the OUT_DIR environment variable
        // Because of (1), we can't use the WASI SDK to build QuickJS so instead we use the system
        // llvm. To compile WASI with the system llvm, we need a WASI sysroot and to install a
        // libclang_rt builtins archive for WASI.
        // The WASI sysroot is provided by the preinstalled `wasi-libc` Ubuntu package on docs.rs.
        // The clang runtime builtins archive, which we vendor in this crate, needs to copied into
        // one of the llvm library directories.
        // Since we can't modify the system llvm library directories, we:
        // 1. Copy the system's llvm installation into the OUT_DIR.
        // 2. Copy the clang runtime builtins archive into that OUT_DIR llvm installation.
        // 3. Use that OUT_DIR llvm installation to compile QuickJS.

        // If errors start occurring pertaining to the compiler complaining about the libclang_rt
        // builtins, you may need to change the version of the of the file we've vendored.
        // The system version of llvm may also need to be changed to match what's in docs.rs.
        let new_llvm_path = copy_system_llvm_to_out_dir()?;
        install_vendored_libclang_rt_builtins(&new_llvm_path)?;
        (
            new_llvm_path.join("bin/clang"),
            new_llvm_path.join("bin/llvm-ar"),
            PathBuf::from("/usr"),
        )
    } else {
        let wasi_sdk_path = get_wasi_sdk_path().await?;
        if !wasi_sdk_path.try_exists()? {
            return Err(anyhow!(
                "wasi-sdk not installed in specified path of {}",
                wasi_sdk_path.display()
            ));
        }
        (
            wasi_sdk_path.join("bin/clang"),
            wasi_sdk_path.join("bin/ar"),
            wasi_sdk_path.join("share/wasi-sysroot"),
        )
    };
    env::set_var("CC", clang_path.to_str().unwrap());
    env::set_var("AR", ar_path.to_str().unwrap());
    let sysroot = format!("--sysroot={}", sysroot.display());
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
        .flag_if_supported("-Wno-implicit-function-declaration")
        .flag_if_supported("-Wno-implicit-const-int-float-conversion")
        .target("wasm32-wasi")
        .opt_level(2)
        .compile("quickjs");

    // Generate bindings for quickjs
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(&["-fvisibility=default", "--target=wasm32-wasi", &sysroot])
        .size_t_is_usize(false)
        .generate()?;

    println!("cargo:rerun-if-changed=extensions/value.c");
    println!("cargo:rerun-if-changed=wrapper.h");

    for entry in WalkDir::new("quickjs") {
        println!("cargo:rerun-if-changed={}", entry?.path().display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_dir.join("bindings.rs"))?;
    Ok(())
}
