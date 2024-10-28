use std::io::{stdout, Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Config;

macro_rules! runtime_config {
    (
        $(#[$attr:meta])*
        pub struct $opts:ident {
            $(
                $(#[doc = $doc:tt])*
                $opt:ident: Option<bool>,
            )+
        }

    ) => {
        $(#[$attr])*
        pub struct $opts {
            $(
                $opt: Option<bool>,
            )+
        }

        impl $opts {
            fn supported_config() -> SupportedConfigProperties {
                SupportedConfigProperties {
                    supported_properties: vec![
                        $(
                            {
                                ConfigProperty {
                                    name: stringify!($opt).replace('_', "-").to_string(),
                                    help: "[=y|n]".to_string(),
                                    doc: concat!($($doc, "\n",)*).into(),
                                }
                            },
                        )+
                    ]
                }
            }
        }
    }
}

runtime_config! {
    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    pub struct SharedConfig {
        /// Whether to redirect the output of console.log to standard error.
        redirect_stdout_to_stderr: Option<bool>,
    }
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
            serde_json::to_string(&SharedConfig::supported_config())
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
        config.redirect_stdout_to_stderr(value);
    }
    Ok(config)
}
