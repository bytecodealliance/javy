use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};
use std::path::Path;

use crate::{js::JS, wit};

pub struct Export {
    pub wit: String,
    pub js: String,
}

pub fn process_exports(js: &JS, wit: &Path, wit_world: &str) -> Result<Vec<Export>> {
    let js_exports = js.exports()?;
    wit::parse_exports(wit, wit_world)?
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
