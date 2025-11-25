use anyhow::{Result, anyhow};
use convert_case::{Case, Casing};
use std::path::Path;

use crate::js::JS;
use crate::wit;

pub(crate) type Exports = Vec<Export>;

#[derive(Debug, Clone)]
pub(crate) struct Export {
    pub wit: String,
    pub js: String,
}

pub(crate) fn process_exports(js: &JS, wit: &Path, wit_world: &str) -> Result<Vec<Export>> {
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
