use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup(fixture_name: &str) -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let crates_dir = temp_dir.path().join("crates").join("test-crate");
    fs::create_dir_all(&crates_dir)?;

    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture_name);

    fs::copy(
        fixtures_dir.join("Cargo.toml"),
        crates_dir.join("Cargo.toml"),
    )?;

    let changelog = fixtures_dir.join("CHANGELOG.md");
    if changelog.exists() {
        fs::copy(changelog, crates_dir.join("CHANGELOG.md"))?;
    }

    Ok(temp_dir)
}

#[test]
fn test_set_release_versions() -> Result<()> {
    let root = setup("with-alpha")?;

    let mut crates =
        javy_release::PublishableCrates::with_root(&["test-crate".to_string()], Some(root.path()))?;
    crates.set_release_versions()?;

    let cargo_toml = fs::read_to_string(root.path().join("crates/test-crate/Cargo.toml"))?;

    assert!(cargo_toml.contains(r#"version = "1.0.0""#));
    assert!(!cargo_toml.contains("-alpha"));

    let changelog = fs::read_to_string(root.path().join("crates/test-crate/CHANGELOG.md"))?;

    assert!(!changelog.contains("## [Unreleased]"));
    assert!(changelog.contains("## [1.0.0] -"));

    Ok(())
}

#[test]
fn test_set_dev_versions() -> Result<()> {
    let root = setup("without-alpha")?;

    let mut crates =
        javy_release::PublishableCrates::with_root(&["test-crate".to_string()], Some(root.path()))?;
    crates.set_dev_versions()?;

    let cargo_toml = fs::read_to_string(root.path().join("crates/test-crate/Cargo.toml"))?;

    assert!(cargo_toml.contains(r#"version = "1.0.1-alpha.1""#));

    Ok(())
}

#[test]
fn test_skips_unpublishable_crates() -> Result<()> {
    let root = TempDir::new()?;
    let crates_dir = root.path().join("crates").join("unpublishable");
    fs::create_dir_all(&crates_dir)?;

    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unpublishable");

    fs::copy(
        fixtures_dir.join("Cargo.toml"),
        crates_dir.join("Cargo.toml"),
    )?;

    let crates = javy_release::PublishableCrates::with_root(
        &["unpublishable".to_string()],
        Some(root.path()),
    )?;

    assert!(crates.is_empty());

    Ok(())
}

#[test]
fn test_preserves_toml_formatting() -> Result<()> {
    let root = setup("with-alpha")?;

    let mut crates =
        javy_release::PublishableCrates::with_root(&["test-crate".to_string()], Some(root.path()))?;
    crates.set_release_versions()?;

    let cargo_toml = fs::read_to_string(root.path().join("crates/test-crate/Cargo.toml"))?;

    assert!(cargo_toml.contains(r#"name = "test-crate""#));
    assert!(cargo_toml.contains("[dependencies]"));

    Ok(())
}

#[test]
fn test_skips_crate_without_alpha_on_set_release_versions() -> Result<()> {
    let root = setup("without-alpha")?;

    let original_cargo_toml = fs::read_to_string(root.path().join("crates/test-crate/Cargo.toml"))?;

    let mut crates =
        javy_release::PublishableCrates::with_root(&["test-crate".to_string()], Some(root.path()))?;
    crates.set_release_versions()?;

    let cargo_toml = fs::read_to_string(root.path().join("crates/test-crate/Cargo.toml"))?;

    assert_eq!(original_cargo_toml, cargo_toml);
    Ok(())
}

#[test]
fn test_handles_missing_crates_gracefully() -> Result<()> {
    let root = TempDir::new().unwrap();

    fs::create_dir_all(root.path().join("crates"))?;

    let crates = javy_release::PublishableCrates::with_root(
        &["non-existent".to_string()],
        Some(root.path()),
    )?;

    assert!(crates.is_empty());

    Ok(())
}
