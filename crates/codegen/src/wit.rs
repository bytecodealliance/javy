use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use wit_parser::{Resolve, WorldItem};

/// Options for using WIT in the code generation process.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct WitOptions {
    /// The path of the .wit file to use.
    pub path: Option<PathBuf>,
    /// The name of the wit world to use.
    pub world: Option<String>,
}

impl WitOptions {
    /// Generate WitOptions from a Tuple of Options.
    pub fn from_tuple(opts: (Option<PathBuf>, Option<String>)) -> Result<Self> {
        match opts {
            (None, None) => Ok(Self {
                path: None,
                world: None,
            }),
            (None, Some(_)) => Ok(Self {
                path: None,
                world: None,
            }),
            (Some(_), None) => bail!("Must provide WIT world when providing WIT file"),
            (path, world) => Ok(Self { path, world }),
        }
    }

    /// Whether WIT options were defined.
    pub(crate) fn defined(&self) -> bool {
        self.path.is_some() && self.world.is_some()
    }

    /// Unwraps a refernce to the .wit file path.
    pub(crate) fn unwrap_path(&self) -> &PathBuf {
        self.path.as_ref().unwrap()
    }

    /// Unwraps a reference to the WIT world name.
    pub(crate) fn unwrap_world(&self) -> &String {
        self.world.as_ref().unwrap()
    }
}

pub(crate) fn parse_exports(wit: impl AsRef<Path>, world: &str) -> Result<Vec<String>> {
    let mut resolve = Resolve::default();
    resolve.push_path(wit.as_ref())?;
    let (_, package_id) = resolve.package_names.first().unwrap();
    let world_id = resolve.select_world(&[*package_id], Some(world))?;
    let world = resolve.worlds.get(world_id).unwrap();

    if !world.imports.is_empty() {
        bail!("Imports in WIT file are not supported");
    }
    let mut exported_functions = vec![];
    for (_, export) in &world.exports {
        match export {
            WorldItem::Interface { .. } => {
                bail!("Exported interfaces are not supported")
            }
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
