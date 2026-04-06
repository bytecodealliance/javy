//! Release automation for all publishable crates in the workspace.
//!
//! This utility automates parts of the versioning process when creating
//! new releases—specifically, setting development and release versions.
//!
//! This utility **does not**:
//!   - Automatically bump versions. Bumps must be done manually following semver.
//!   - Automatically publish to crates.io.

use javy_release::PublishableCrates;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <action> <crate1> [crate2] ...", args[0]);
        eprintln!("Actions:");
        eprintln!("  set-release-versions - Remove alpha suffixes and update changelogs");
        eprintln!("  set-dev-versions     - Bump versions and add alpha suffix");
        process::exit(1);
    }

    let action = &args[1];
    let crate_names = &args[2..];

    let mut publishable_crates = PublishableCrates::new(crate_names)?;

    if publishable_crates.is_empty() {
        eprintln!("No publishable crates found");
        process::exit(1);
    }

    match action.as_str() {
        "set-release-versions" => publishable_crates.set_release_versions()?,
        "set-dev-versions" => publishable_crates.set_dev_versions()?,
        _ => {
            eprintln!("Unknown action: {}", action);
            eprintln!("Valid actions: set-release-versions, publish, set-dev-versions");
            process::exit(1);
        }
    }

    Ok(())
}
