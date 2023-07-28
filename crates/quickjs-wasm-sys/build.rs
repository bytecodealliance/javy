use anyhow::{anyhow, Result};
use hyper::body::Incoming;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs, process};

use http_body_util::BodyExt;
use hyper::{body::Buf, Uri};
use tokio::io::{AsyncRead, AsyncWrite};

use walkdir::WalkDir;

const WASI_SDK_VERSION_MAJOR: usize = 20;
const WASI_SDK_VERSION_MINOR: usize = 0;

async fn tls_connect(url: &Uri) -> Result<impl AsyncRead + AsyncWrite + Unpin> {
    let connector: tokio_native_tls::TlsConnector =
        tokio_native_tls::native_tls::TlsConnector::new()
            .unwrap()
            .into();
    let addr = format!("{}:{}", url.host().unwrap(), url.port_u16().unwrap_or(443));
    let stream = tokio::net::TcpStream::connect(addr).await?;
    let stream = connector.connect(url.host().unwrap(), stream).await?;
    Ok(stream)
}

// Mostly taken from the hyper examples:
// https://github.com/hyperium/hyper/blob/4cf38a12ce7cc5dfd3af356a0cef61ace4ce82b9/examples/client.rs
async fn get_uri(url_str: impl AsRef<str>) -> Result<Incoming> {
    let mut url_string = url_str.as_ref().to_string();
    // This loop will follow redirects and will return when a status code
    // is a success (200-299) or a non-redirect (300-399).
    loop {
        let url: Uri = url_string.parse()?;
        let stream = tls_connect(&url).await?;
        let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let authority = url.authority().unwrap().clone();
        let req = hyper::Request::builder()
            .uri(&url)
            .header(hyper::header::HOST, authority.as_str())
            .body("".to_string())?;

        let res = sender.send_request(req).await?;
        if res.status().is_success() {
            return Ok(res.into_body());
        } else if res.status().is_redirection() {
            let target = res
                .headers()
                .get("Location")
                .ok_or(anyhow!("Redirect without `Location` header"))?;
            url_string = target.to_str()?.to_string();
        } else {
            return Err(anyhow!("Could not request URL {:?}", url));
        }
    }
}

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

        let uri = format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{major_version}/wasi-sdk-{major_version}.{minor_version}-{file_suffix}.tar.gz");
        let mut body = get_uri(uri).await?;
        let mut archive = fs::File::create(&archive_path)?;
        while let Some(frame) = body.frame().await {
            if let Some(chunk) = frame
                .map_err(|err| {
                    anyhow!(
                        "Something went wrong when downloading the WASI SDK: {}",
                        err
                    )
                })?
                .data_ref()
            {
                archive.write_all(chunk.chunk())?;
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let wasi_sdk_path = get_wasi_sdk_path().await?;
    if !wasi_sdk_path.try_exists()? {
        return Err(anyhow!(
            "wasi-sdk not installed in specified path of {}",
            wasi_sdk_path.display()
        ));
    }
    env::set_var("CC", format!("{}/bin/clang", wasi_sdk_path.display()));
    env::set_var("AR", format!("{}/bin/ar", wasi_sdk_path.display()));
    let sysroot = format!("--sysroot={}/share/wasi-sysroot", wasi_sdk_path.display());
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
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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
