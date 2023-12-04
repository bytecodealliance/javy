use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};
use std::{env, path::Path};

use crate::{js::JS, wit};

pub struct Export {
    pub wit: String,
    pub js: String,
}

pub fn process_exports(js: &JS, wit: &Path, wit_world: &str) -> Result<Vec<Export>> {
    let js_exports = js.exports()?;
    parse_wit_exports(wit, wit_world)?
        .into_iter()
        .map(|wit_export| {
            let export = wit_export.from_case(Case::Kebab).to_case(Case::Camel);
            if !js_exports.contains(&export) {
                Err(anyhow!("JS module does not export {export}"))
            } else {
                Ok(Export {
                    wit: wit_export,
                    js: export,
                })
            }
        })
        .collect::<Result<Vec<Export>>>()
}

fn parse_wit_exports(wit: &Path, wit_world: &str) -> Result<Vec<String>> {
    // Configure wit-parser to not require semicolons but only if the relevant
    // environment variable is not already set.
    const SEMICOLONS_OPTIONAL_ENV_VAR: &str = "WIT_REQUIRE_SEMICOLONS";
    let semicolons_env_var_already_set = env::var(SEMICOLONS_OPTIONAL_ENV_VAR).is_ok();
    if !semicolons_env_var_already_set {
        env::set_var(SEMICOLONS_OPTIONAL_ENV_VAR, "0");
    }

    let exports = wit::parse_exports(wit, wit_world);

    // If we set the environment variable to not require semicolons, remove
    // that environment variable now that we no longer need it set.
    if !semicolons_env_var_already_set {
        env::remove_var(SEMICOLONS_OPTIONAL_ENV_VAR);
    }

    exports
}
