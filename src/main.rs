mod cli;
mod core;
mod doctor;
mod registry;
mod spec;
mod trust;

use anyhow::Result;

fn main() -> Result<()> {
    let command = cli::parse();
    cli::commands::run(command)
}
