use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    /// Update all package
    #[arg(short, long)]
    pub all: bool,

    /// Update git packages
    #[arg(short, long)]
    pub git: bool,

    /// Only list outdated packages
    #[arg(short, long)]
    pub list: bool,

    /// Package names to update
    ///
    /// This is roughly equivalent to `cargo install -f <PACKAGES>`
    #[arg(id = "PACKAGES")]
    pub names: Option<Vec<String>>,
}
