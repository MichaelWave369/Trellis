use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "trellis",
    version,
    about = "Local-first package manager prototype"
)]
pub struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    pub home: Option<std::path::PathBuf>,

    #[arg(long, global = true, value_name = "PATH")]
    pub registry_root: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init,
    Install { pkg: String },
    Remove { pkg: String },
    Update,
    List,
    Search { query: String },
    Info { pkg: String },
    Doctor,
}

pub fn parse() -> Cli {
    Cli::parse()
}

pub mod commands;
