use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "trellis",
    version,
    about = "Registry-driven local-first package manager",
    long_about = "Trellis v0.5 adds a cohesive CLI UX layer with clearer status output, install resolution summaries, and human-readable receipt rendering."
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
    /// Refresh active registry indexes and materialize local cache state
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
    /// Render an installed package receipt in human-readable form
    Receipt { pkg: String },
    /// Run environment and state integrity checks
    Doctor,
}

pub fn parse() -> Cli {
    Cli::parse()
}

pub mod commands;
pub mod ui;
