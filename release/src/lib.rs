use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{value, DocumentMut};

pub struct PublishableCrate {
    pub name: String,
    pub cargo_toml_path: PathBuf,
    pub cargo_toml_doc: DocumentMut,
}

pub struct PublishableCrates {
    pub crates: Vec<PublishableCrate>,
}

impl PublishableCrate {
    fn remove_alpha_suffix(&mut self) -> Option<String> {
        if let Some(package) = self.cargo_toml_doc.get("package") {
            if let Some(version_item) = package.get("version") {
                if let Some(version_str) = version_item.as_str() {
                    if version_str.contains("-alpha") {
                        let new_version = version_str.split("-alpha").next().unwrap().to_string();
                        if let Some(package_mut) = self.cargo_toml_doc.get_mut("package") {
                            package_mut["version"] = value(&new_version);
                        }
                        return Some(new_version);
                    }
                }
            }
        }
        None
    }

    fn add_alpha_suffix(&mut self) {
        if let Some(package) = self.cargo_toml_doc.get_mut("package") {
            if let Some(version_item) = package.get("version") {
                if let Some(version_str) = version_item.as_str() {
                    if !version_str.contains("-alpha") {
                        let new_version = bump_patch_and_add_alpha(version_str)
                            .unwrap_or_else(|| format!("{}-alpha.1", version_str));
                        package["version"] = value(&new_version);
                    }
                }
            }
        }
    }

    fn save(&self) -> Result<()> {
        Ok(fs::write(
            &self.cargo_toml_path,
            self.cargo_toml_doc.to_string(),
        )?)
    }

    fn update_changelog(&self, version: &str, date: &str) -> Result<()> {
        let changelog_path = self
            .cargo_toml_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not get parent directory of Cargo.toml"))?
            .join("CHANGELOG.md");

        if !changelog_path.exists() {
            return Ok(());
        }

        let changelog_content = fs::read_to_string(&changelog_path)?;
        let updated_changelog =
            changelog_content.replace("## [Unreleased]", &format!("## [{}] - {}", version, date));

        fs::write(&changelog_path, updated_changelog)?;
        println!(
            "Updated {} with version {} and date {}",
            changelog_path.display(),
            version,
            date
        );

        Ok(())
    }
}

impl PublishableCrates {
    pub fn new(crate_names: &[String]) -> Result<Self> {
        Self::with_root(crate_names, None)
    }

    pub fn with_root(crate_names: &[String], root: Option<&Path>) -> Result<Self> {
        let root = match root {
            Some(r) => r.to_path_buf(),
            None => std::env::current_dir()?,
        };
        let mut crates = Vec::new();

        for name in crate_names {
            let dir_name = name.strip_prefix("javy-").unwrap_or(name);
            let cargo_toml_path = root.join("crates").join(dir_name).join("Cargo.toml");

            if !cargo_toml_path.exists() {
                eprintln!("Warning: {} not found, skipping", cargo_toml_path.display());
                continue;
            }

            let cargo_toml_content = match fs::read_to_string(&cargo_toml_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!(
                        "Error reading {}: {}, skipping",
                        cargo_toml_path.display(),
                        e
                    );
                    continue;
                }
            };

            let cargo_toml_doc: DocumentMut = match cargo_toml_content.parse() {
                Ok(doc) => doc,
                Err(e) => {
                    eprintln!(
                        "Error parsing TOML for {}: {}, skipping",
                        cargo_toml_path.display(),
                        e
                    );
                    continue;
                }
            };

            if !is_publishable(&cargo_toml_doc) {
                println!("Skipping {} (publish = false)", name);
                continue;
            }

            crates.push(PublishableCrate {
                name: name.clone(),
                cargo_toml_path,
                cargo_toml_doc,
            });
        }

        Ok(PublishableCrates { crates })
    }

    pub fn is_empty(&self) -> bool {
        self.crates.is_empty()
    }

    pub fn set_release_versions(&mut self) -> Result<()> {
        let today = Local::now().format("%Y-%m-%d").to_string();

        for krate in &mut self.crates {
            let version = krate.remove_alpha_suffix();

            if let Some(version) = version {
                krate.save()?;

                println!("Updated {} to version {}", krate.name, version);

                krate.update_changelog(&version, &today)?;
            } else {
                println!("No alpha suffix found in {}, skipping", krate.name);
            }
        }

        println!("Release versions set successfully");
        println!("\nNext steps:");
        println!("1. Make a PR with these changes");
        println!("2. Publish the new versions to crates.io");
        println!("3. Run `./scripts/release.sh set-dev-versions`");
        Ok(())
    }

    pub fn set_dev_versions(&mut self) -> Result<()> {
        for krate in &mut self.crates {
            krate.add_alpha_suffix();

            krate.save()?;

            println!("Updated {} with alpha suffix", krate.name);
        }

        println!("\nDev versions set successfully!");
        Ok(())
    }
}

fn bump_patch_and_add_alpha(version: &str) -> Option<String> {
    let parts: Vec<&str> = version.splitn(3, '.').collect();
    if parts.len() != 3 {
        return None;
    }
    let patch: u64 = parts[2].parse().ok()?;
    Some(format!("{}.{}.{}-alpha.1", parts[0], parts[1], patch + 1))
}

fn is_publishable(doc: &DocumentMut) -> bool {
    match doc.get("package") {
        Some(p) => {
            match p.get("publish") {
                // The key **must** be a boolean and guaranteed by cargo.
                Some(v) => v.as_bool().unwrap(),
                // No explicit `publish` key is publishable by
                // default.
                None => true,
            }
        }
        // No package to publish.
        None => false,
    }
}
