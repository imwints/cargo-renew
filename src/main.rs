mod cli;
mod config;
mod package;
mod registry;

use std::process::Command;

use crate::cli::Args;
use crate::config::get_cargo_install_root;
use crate::package::{find_installed_packages, has_update, Package, Registry};

use anyhow::{bail, Result};
use clap::Parser;
use owo_colors::{OwoColorize, Style};
use reqwest::blocking::Client;

fn main() -> Result<()> {
    let _args = Args::parse();

    if _args.git {
        bail!("Not implemented yet!")
    }

    let cargo_install_root = get_cargo_install_root()?;
    let crates_file = cargo_install_root.join(".crates.toml");
    if !crates_file.exists() {
        return Ok(());
    }

    let packages: Vec<Package> = find_installed_packages(&crates_file)?
        .into_iter()
        .filter(|package| match package.registry {
            Registry::Registry(_) => true,
            _ => _args.git,
        })
        .collect();

    let client = Client::new();
    let packages: Vec<Package> = packages
        .into_iter()
        .filter_map(|package| crate::package::pull_version(package, Some(&client)))
        .collect();

    print_update_table(&packages);

    if _args.list {
        return Ok(());
    }

    for package in packages {
        if !has_update(&package) {
            continue;
        }

        match package.registry {
            Registry::Git { url, commit: _ } => Command::new("cargo")
                .arg("install")
                .arg("--git")
                .arg(url)
                .arg(package.name)
                .spawn()?
                .wait()?,
            Registry::Registry(_) => Command::new("cargo")
                .arg("install")
                .arg(package.name)
                .spawn()?
                .wait()?,
        };
    }

    Ok(())
}

fn print_update_table(packages: &Vec<Package>) {
    if packages.is_empty() {
        return;
    }

    let package_width = packages
        .iter()
        .map(|package| package.name.len())
        .max()
        .unwrap_or(12);

    println!(
        "{:>12} {:<package_width$} {:<12} {:<12}",
        "Status".bold().green(),
        "Package".bold(),
        "Version".bold(),
        "Latest".bold(),
    );

    for package in packages {
        let update = has_update(&package);
        let prefix = if update { "Update" } else { "Fresh" };
        let style = if update {
            Style::new().bold().yellow()
        } else {
            Style::new()
        };

        print!(
            "{:>12} {:<package_width$} {:<12} {:<12}",
            prefix.bold().green(),
            package.name,
            package.version,
            package.latest_version.as_ref().unwrap().style(style),
        );
    }
}
