use std::path::PathBuf;

use anyhow::Result;
use javy_codegen::{Generator, LinkingKind, Plugin, JS};

#[test]
fn test_empty() -> Result<()> {
    // Load valid JS from file.
    let js = JS::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("sample-scripts")
            .join("empty.js")
            .as_path(),
    )?;

    // Load existing pre-initialized Javy plugin.
    let plugin =
        Plugin::new_from_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_plugin.wasm"))?;

    // Configure code generator.
    let mut generator = Generator::new(plugin);
    generator.linking(LinkingKind::Static);

    // Generate valid WASM module.
    generator.generate(&js)?;

    Ok(())
}
