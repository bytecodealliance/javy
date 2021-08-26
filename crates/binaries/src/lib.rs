use anyhow::{anyhow, bail, Result};
use binary_install::Cache;
use std::fmt;
use std::path::PathBuf;

const CACHE_NAME: &str = "javy";

pub enum Binary {
    WasmOpt,
    WasmStrip,
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Binary::WasmStrip => "wasm-strip",
            Binary::WasmOpt => "wasm-opt",
        };
        write!(f, "{}", s)
    }
}

pub fn wasm_opt() -> Result<PathBuf> {
    let name = Binary::WasmOpt.to_string();
    let (libdir, dylibname) = dylibinfo(Binary::WasmOpt)?;

    match which::which(&name) {
        Ok(cmd) => Ok(cmd),
        _ => {
            let cache = Cache::new(CACHE_NAME).map_err(anyhow::Error::msg)?;
            let url = download_url(&Binary::WasmOpt)?;
            let download = cache
                .download_artifact(&name, &url)
                .map_err(anyhow::Error::msg)?;

            if let Some(d) = download {
                let root = d.path();
                let wasm_opt = root.join("bin").join(&name);
                let wasm_opt_dylib = root.join(&libdir).join(&dylibname);

                if wasm_opt.exists() && wasm_opt_dylib.exists() {
                    return Ok(wasm_opt);
                }

                if !wasm_opt.exists() {
                    std::fs::create_dir_all(root.join("bin"))?;
                    std::fs::rename(root.join(&name), root.join("bin").join(&name))?;
                }

                if !wasm_opt_dylib.exists() {
                    std::fs::create_dir_all(root.join(&libdir))?;
                    std::fs::rename(root.join(&dylibname), root.join(&libdir).join(&dylibname))?;
                }

                Ok(wasm_opt)
            } else {
                Err(anyhow!("Couldn't find wasm-opt"))
            }
        }
    }
}

pub fn wasm_strip() -> Result<PathBuf> {
    let name = Binary::WasmStrip.to_string();

    match which::which(&name) {
        Ok(cmd) => Ok(cmd),
        _ => {
            let cache = Cache::new(CACHE_NAME).map_err(anyhow::Error::msg)?;
            let url = download_url(&Binary::WasmStrip)?;
            let download = cache
                .download_artifact(&name, &url)
                .map_err(anyhow::Error::msg)?;
            if let Some(d) = download {
                d.binary(&name).map_err(anyhow::Error::msg)
            } else {
                Err(anyhow!("Couldn't find wasm-strip"))
            }
        }
    }
}

fn dylibinfo(bin: Binary) -> Result<(String, String)> {
    match bin {
        Binary::WasmOpt => {
            if cfg!(target_os = "linux") {
                return Ok(("lib64".into(), "libbinaryen.a".into()));
            }

            if cfg!(target_os = "macos") {
                return Ok(("lib".into(), "libbinaryen.dylib".into()));
            }

            bail!("Target architecture not supported")
        }
        _ => Ok(("".into(), "".into())),
    }
}

fn target(bin: &Binary) -> Result<String> {
    match bin {
        Binary::WasmOpt => {
            if cfg!(target_os = "linux") {
                return Ok("x86_64-linux".to_string());
            }

            if cfg!(target_os = "macos") {
                return Ok("x86_64-macos".to_string());
            }

            bail!("Target architecture not supported")
        }

        Binary::WasmStrip => {
            if cfg!(target_os = "linux") {
                return Ok("ubuntu".to_string());
            }

            if cfg!(target_os = "macos") {
                return Ok("macos".to_string());
            }

            bail!("Target architecture not supported");
        }
    }
}

fn download_url(bin: &Binary) -> Result<String> {
    let target = target(bin)?;
    match bin {
        Binary::WasmOpt => Ok(
             format!("https://github.com/WebAssembly/binaryen/releases/download/{v}/binaryen-{v}-{target}.tar.gz", v = "version_101", target = target)
        ),
        Binary::WasmStrip => Ok(
            format!("https://github.com/WebAssembly/wabt/releases/download/{v}/wabt-{v}-{target}.tar.gz", v = "1.0.23", target = target)
        )
    }
}
