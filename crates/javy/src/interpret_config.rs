use std::io::{stdout, Write};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::Config;

// use crate::Config;

// option_group! {
//     #[derive(Clone, Debug)]
//     pub enum JsOption {
//         /// Whether to redirect the output of console.log to standard error.
//         RedirectStdoutToStderr(bool),
//         /// Whether to enable the `Javy.JSON` builtins.
//         JavyJson(bool),
//         /// Whether to enable the `Javy.readSync` and `Javy.writeSync` builtins.
//         JavyStreamIo(bool),
//         /// Whether to override the `JSON.parse` and `JSON.stringify`
//         /// implementations with an alternative, more performant, SIMD based
//         /// implemetation.
//         SimdJsonBuiltins(bool),
//         /// Whether to enable support for the `TextEncoder` and `TextDecoder`
//         /// APIs.
//         TextEncoding(bool),
//     }
// }

// pub fn interpret_config(config: &[&str]) -> Config {
//     todo!()
// }

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct SharedConfig {
    /// Whether to redirect the output of console.log to standard error.
    redirect_stdout_to_stderr: Option<String>,
}

#[derive(Debug, Serialize)]
struct SupportedConfigProperties {
    supported_properties: Vec<ConfigProperty>,
}

#[derive(Debug, Serialize)]
struct ConfigProperty {
    name: String,
    help: String,
    doc: String,
}

#[export_name = "supported_config"]
pub fn supported_config() {
    stdout()
        .write_all(
            serde_json::to_string(&SupportedConfigProperties {
                supported_properties: vec![ConfigProperty {
                    name: "redirect-stdout-to-stderr".to_string(),
                    help: "[=y|n]".to_string(),
                    doc: "Whether to redirect the output of console.log to standard error."
                        .to_string(),
                }],
            })
            .unwrap()
            .as_bytes(),
        )
        .unwrap();
    stdout().flush().unwrap();
}

pub fn parse_config(config: &[u8]) -> Result<Config> {
    let shared_config = serde_json::from_slice::<SharedConfig>(config)?;
    let mut config = Config::default();
    if let Some(value) = shared_config.redirect_stdout_to_stderr {
        if value != "" && value != "y" && value != "n" {
            bail!("Value for `redirect-stdout-to-stderr` must be `y` or `n`");
        }
        config.redirect_stdout_to_stderr(value == "" || value == "y");
    }
    Ok(config)
}
