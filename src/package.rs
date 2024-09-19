use std::{fs::read_to_string, path::Path, str::FromStr};

use anyhow::{Context, Error, Result};
use reqwest::blocking::Client;
use semver::Version;
use toml::Table;

use crate::registry::latest_version;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Registry {
    Git { url: String, commit: String },
    Registry(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Package {
    /// Package name
    pub name: String,

    /// Current package version
    pub version: Version,

    /// Registry url
    pub registry: Registry,

    /// Available versions from the registry
    pub latest_version: Option<Version>,
}

impl FromStr for Registry {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let registry = s
            .strip_prefix("(")
            .context("Missing '(' before registry")?
            .strip_suffix(")")
            .context("Missing ')' after registry")?;

        if let Some(url) = registry.strip_prefix("git+") {
            let (url, commit) = url
                .split_once("#")
                .context("Couldn't split git url from commit hash")?;
            Ok(Self::Git {
                url: String::from(url),
                commit: String::from(commit),
            })
        } else {
            let url = registry
                .strip_prefix("registry+")
                .or_else(|| registry.strip_prefix("sparse+"))
                .context("Invalid registry prefix")?;
            Ok(Self::Registry(String::from(url)))
        }
    }
}

impl FromStr for Package {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.splitn(3, " ");
        let name = parts
            .next()
            .context("Failed to get package name from crates key")?;
        let version = parts
            .next()
            .context("Failed to get package version from crates key")?;
        let registry = parts
            .next()
            .context("Failed to get package registry from crates key")?;
        let version = Version::from_str(version)?;
        let registry = Registry::from_str(registry)?;
        Ok(Self {
            name: String::from(name),
            version,
            registry,
            latest_version: None,
        })
    }
}

pub fn find_installed_packages(crates_file: &Path) -> Result<Vec<Package>> {
    let contents = read_to_string(crates_file)?;
    let root = contents.parse::<Table>()?;
    let Some(v1) = root.get("v1") else {
        anyhow::bail!("No value 'v1' in '.crates.toml'")
    };
    let Some(v1) = v1.as_table() else {
        anyhow::bail!("No table 'v1' in '.crates.toml'")
    };
    let packages: Vec<Package> = v1
        .into_iter()
        .map(|(k, _)| k)
        .map(|package| Package::from_str(package))
        .filter_map(Result::ok)
        .collect();
    Ok(packages)
}

pub fn pull_version(package: Package, client: Option<&Client>) -> Option<Package> {
    let Package {
        name,
        version,
        registry,
        latest_version: _,
    } = package;
    let latest_version = latest_version(&name, client).ok()?;
    Some(Package {
        name,
        version,
        registry,
        latest_version: Some(latest_version),
    })
}

/// If the package has it's latest_version field populated, return latest > version
pub fn has_update(package: &Package) -> bool {
    if let Some(latest_version) = &package.latest_version {
        latest_version > &package.version
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_to_package() {
        let Package {
            name,
            version,
            registry,
            latest_version: _,
        } = Package::from_str("bat 0.24.0 (registry+https://github.com/rust-lang/crates.io-index)")
            .unwrap();
        assert_eq!(name, "bat");
        assert_eq!(version, Version::new(0, 24, 0));
        assert_eq!(
            registry,
            Registry::Registry(String::from("https://github.com/rust-lang/crates.io-index"))
        );
    }

    #[test]
    fn test_git_to_package() {
        let Package {
            name, version, registry, latest_version: _
        } = Package::from_str("ruff 0.6.4 (git+https://github.com/astral-sh/ruff#43a5922f6f11784f74e6f553467b1be802bc2213)").unwrap();
        assert_eq!(name, "ruff");
        assert_eq!(version, Version::new(0, 6, 4));
        assert_eq!(
            registry,
            Registry::Git {
                url: String::from("https://github.com/astral-sh/ruff"),
                commit: String::from("43a5922f6f11784f74e6f553467b1be802bc2213")
            }
        );
    }
}
