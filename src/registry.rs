use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;

const CRATES_IO_INDEX: &str = "https://index.crates.io";

/// Represenation of a line from a packages index file from index.crates.io
///
/// Adapted from `cargo::sources::registry::index::IndexPackage`
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct IndexPackage {
    /// Name of the package
    pub name: String,
    /// Version of the package
    pub vers: Version,
}

fn available_versions(name: &str, client: Option<&Client>) -> Result<Vec<Version>> {
    let url = format!("{}/{}", CRATES_IO_INDEX, make_index_file(name));
    let response = if let Some(client) = client {
        client.get(url).send()?
    } else {
        reqwest::blocking::get(url)?
    };
    let index = response.text()?;
    let versions: Vec<Version> = index
        .split("\n")
        .filter_map(|package| serde_json::from_str::<IndexPackage>(package).ok())
        .map(|package| package.vers)
        .collect();
    Ok(versions)
}

pub fn latest_version(name: &str, client: Option<&Client>) -> Result<Version> {
    let versions = available_versions(name, client)?;
    versions
        .last()
        .map(|version| version.to_owned())
        .ok_or(anyhow!("No version found"))
}

/// Create an index file based on a package name
fn make_index_file(name: &str) -> String {
    match name.len() {
        1 => format!("1/{}", name),
        2 => format!("2/{}", name),
        3 => format!("3/{}/{}", &name[..1], name),
        _ => format!("{}/{}/{}", &name[..2], &name[2..4], name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_files() {
        assert_eq!("1/b", make_index_file("b"));
        assert_eq!("2/xy", make_index_file("xy"));
        assert_eq!("2/x-", make_index_file("x-"));
        assert_eq!("3/x/xy7", make_index_file("xy7"));
        assert_eq!("3/x/xy-", make_index_file("xy-"));
        assert_eq!("el/se/else", make_index_file("else"));
        assert_eq!("lo/ng/long-crate", make_index_file("long-crate"));
    }
}
