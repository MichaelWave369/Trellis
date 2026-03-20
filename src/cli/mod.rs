use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "trellis",
    version,
    about = "Registry-driven local-first package manager",
    long_about = "Trellis v1.0.0-rc1 hardens the existing command surface with coherent local-first workflows, deterministic installs, and honest trust/provenance reporting."
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

    #[arg(
        long,
        global = true,
        default_value = "default",
        value_name = "NAME",
        help = "Environment profile (default, dev, minimal, diagnostics)"
    )]
    pub profile: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create Trellis home directories
    Init,
    /// Install a package by name or directly from a spec path
    Install {
        /// Package name present in active registry index
        pkg: Option<String>,
        #[arg(
            long,
            value_name = "PATH",
            help = "Install directly from a .trellis.yaml path"
        )]
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
    /// Scaffold a new package authoring workspace
    Scaffold {
        package_name: String,
        #[arg(long, default_value = "binary", value_parser = ["binary", "source"]) ]
        kind: String,
        #[arg(long, value_name = "PATH")]
        out: Option<std::path::PathBuf>,
    },
    /// Print author/maintainer submission readiness hints for a spec or package
    Readiness { target: String },
    /// Guided first-run onboarding flow
    Seed,
    /// Alias for `seed`
    Bootstrap,
    /// Verify installed state against receipts and lock state
    Verify,
    /// Attempt to repair exposed binaries from receipt state
    Repair,
    /// Run environment and state integrity checks
    Doctor,
}

pub fn parse() -> Cli {
    Cli::parse()
}

pub mod commands;
pub mod ui;
