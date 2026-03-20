use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "trellis",
    version,
    about = "Local-first package manager prototype",
    long_about = "Trellis v0.2 adds package authoring workflows: spec validation, package inspection, and local install from path."
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
    /// Install a package by name or directly from a spec path
    Install {
        pkg: Option<String>,
        #[arg(long, value_name = "PATH")]
        from: Option<std::path::PathBuf>,
    },
    /// Remove an installed package
    Remove { pkg: String },
    /// Refresh local registry index from filesystem specs
    Update,
    /// List installed packages from receipts
    List,
    /// Search package names and descriptions
    Search { query: String },
    /// Show package metadata from the registry or a spec path
    Info { pkg: String },
    /// Validate a package spec path or package name
    Validate { target: String },
    /// Inspect a package spec path or package name
    Inspect { target: String },
    /// Run environment and state integrity checks
    Doctor,
}

pub fn parse() -> Cli {
    Cli::parse()
}

pub mod commands;
