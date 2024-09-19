use std::{
    env,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

/// Mapping for `config.toml`
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ConfigRoot {
    install: Option<InstallConfig>,
}

/// Mapping for `install.root`
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InstallConfig {
    root: Option<String>,
}

/// Find the cargo home directory
///
/// This is either the value of $CARGO_HOME
/// or $HOME/.cargo
pub fn get_cargo_home() -> Result<PathBuf> {
    match env::var("CARGO_HOME") {
        Ok(env) => Ok(PathBuf::from(env)),
        Err(_) => Ok(dirs::home_dir()
            .ok_or(anyhow!("No home directory found"))?
            .join(".cargo")),
    }
}

/// Find the cargo install directory
///
/// This is either $CARGO_INSTALL_ROOT or
/// `install.root` from $CARGO_HOME/config.toml if found,
// or else $CARGO_HOME
pub fn get_cargo_install_root() -> Result<PathBuf> {
    match env::var("CARGO_INSTALL_ROOT") {
        Ok(env) => Ok(PathBuf::from(env)),
        Err(_) => {
            let cargo_home = get_cargo_home()?;
            let cargo_config = cargo_home.join("config.toml");
            read_install_root(&cargo_config).or_else(|_| Ok(cargo_home))
        }
    }
}

fn read_install_root(path: &Path) -> Result<PathBuf> {
    let contents = read_to_string(path)?;
    let root = toml::from_str::<ConfigRoot>(&contents)?;
    let home_dir = dirs::home_dir().context("No home directory found")?;
    root.install
        .and_then(|install| install.root)
        .map(|root| home_dir.join(root))
        .ok_or(anyhow!("`install.root` not present"))
}
