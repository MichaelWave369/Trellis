use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "trellis",
    version,
    about = "Local-first package manager prototype",
    long_about = "Trellis v0.1 is a local-first package manager prototype with deterministic state, filesystem registry indexing, and auditable install receipts."
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Override Trellis home directory (default: XDG data home fallback)"
    )]
    pub home: Option<std::path::PathBuf>,

    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Override registry root to scan for *.trellis.yaml specs (default: ./packages)"
    )]
    pub registry_root: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create Trellis home directories
    Init,
    /// Install a package from the local registry by exact package name
    Install { pkg: String },
    /// Remove an installed package
    Remove { pkg: String },
    /// Refresh local registry index from filesystem specs
    Update,
    /// List installed packages from receipts
    List,
    /// Search package names and descriptions
    Search { query: String },
    /// Show package metadata from the registry
    Info { pkg: String },
    /// Run environment and state integrity checks
    Doctor,
}

pub fn parse() -> Cli {
    Cli::parse()
}

pub mod commands;
