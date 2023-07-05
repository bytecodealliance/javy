use std::path::Path;

use anyhow::{bail, Result};

use wit_parser::{Resolve, UnresolvedPackage, WorldItem};

pub fn parse_exports(wit: impl AsRef<Path>, world: &str) -> Result<Vec<String>> {
    let mut resolve = Resolve::default();
    let package = UnresolvedPackage::parse_path(wit.as_ref())?;
    resolve.push(package)?;
    let (_, package_id) = resolve.package_names.first().unwrap();
    let world_id = resolve.select_world(*package_id, Some(world))?;
    let world = resolve.worlds.get(world_id).unwrap();

    if !world.imports.is_empty() {
        bail!("Imports in WIT file are not supported");
    }
    let mut exported_functions = vec![];
    for (_, export) in &world.exports {
        match export {
            WorldItem::Interface(_) => bail!("Exported interfaces are not supported"),
            WorldItem::Function(f) => {
                if !f.params.is_empty() {
                    bail!("Exported functions with parameters are not supported")
                } else if f.results.len() != 0 {
                    bail!("Exported functions with return values are not supported")
                } else {
                    exported_functions.push(f.name.clone())
                }
            }
            WorldItem::Type(_) => bail!("Exported types are not supported"),
        }
    }
    Ok(exported_functions)
}
