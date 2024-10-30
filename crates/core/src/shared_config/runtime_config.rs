use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConfigSchema {
    pub(super) supported_properties: Vec<ConfigProperty>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConfigProperty {
    pub(super) name: String,
    pub(super) doc: String,
}

#[macro_export]
macro_rules! runtime_config {
    (
        $(#[$attr:meta])*
        pub struct $opts:ident {
            $(
                $(
                    #[doc = $doc:tt]
                )*
                $opt:ident: Option<bool>,
            )+
        }
    ) => {
        $(#[$attr])*
        pub struct $opts {
            $(
                $(
                    #[doc = $doc]
                )*
                $opt: Option<bool>,
            )+
        }

        impl $opts {
            fn config_schema() -> $crate::shared_config::runtime_config::ConfigSchema {
                $crate::shared_config::runtime_config::ConfigSchema {
                    supported_properties: vec![
                        $(
                            {
                                $crate::shared_config::runtime_config::ConfigProperty {
                                    name: stringify!($opt).replace('_', "-").to_string(),
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
