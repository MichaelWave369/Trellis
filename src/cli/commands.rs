use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::{Cli, Command};
use crate::core::install;
use crate::core::paths::TrellisPaths;
use crate::core::{remove, state};
use crate::doctor::checks;
use crate::registry::index::{read_index, scan_registry};
use crate::registry::sync::sync_registry;

pub fn run(cli: Cli) -> Result<()> {
    let paths = TrellisPaths::resolve(cli.home.as_deref())?;
    let registry_root = resolve_registry_root(cli.registry_root.as_deref())?;

    match cli.command {
        Command::Init => {
            state::init(&paths)?;
            println!("Initialized Trellis home at {}", paths.home.display());
        }
        Command::Update => {
            ensure_initialized(&paths)?;
            let count = sync_registry(&paths, &registry_root)?;
            println!("Updated registry index ({} package(s))", count);
        }
        Command::Search { query } => {
            ensure_initialized(&paths)?;
            ensure_index(&paths, &registry_root)?;
            let query_lower = query.to_lowercase();
            let index = read_index(&paths.registry.join("index.json"))?;
            println!("Search results");

            let mut matched = 0usize;
            for pkg in index.packages.iter().filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
            }) {
                println!("- {:<20} {}", pkg.name, pkg.description);
                matched += 1;
            }

            if matched == 0 {
                println!("(no matches)");
            }
        }
        Command::Info { pkg } => {
            ensure_initialized(&paths)?;
            let entry = find_package(&paths, &registry_root, &pkg)?;
            let spec = entry.spec;
            println!("Package: {}", spec.name);
            println!("Version: {}", spec.version);
            println!("Description: {}", spec.description);
            println!("Homepage: {}", spec.homepage);
            println!("Source: {} ({})", spec.source.path, spec.source.source_type);
            println!("Registry: {}", spec.provenance.registry);
            println!("Publisher: {}", spec.provenance.publisher);
            println!("License: {}", spec.provenance.license);
        }
        Command::Install { pkg } => {
            ensure_initialized(&paths)?;
            let entry = find_package(&paths, &registry_root, &pkg)?;
            install::install(&paths, &entry, &entry.spec)?;
            println!("Installed {} {}", entry.spec.name, entry.spec.version);
        }
        Command::Remove { pkg } => {
            ensure_initialized(&paths)?;
            remove::remove(&paths, &pkg)?;
            println!("Removed {}", pkg);
        }
        Command::List => {
            ensure_initialized(&paths)?;
            println!("Installed packages");
            let mut found = 0usize;
            for entry in fs::read_dir(&paths.receipts)? {
                let entry = entry?;
                if entry.path().extension().and_then(|v| v.to_str()) == Some("json") {
                    let receipt = crate::core::receipts::read_receipt(&entry.path())?;
                    println!("- {:<20} {}", receipt.name, receipt.version);
                    found += 1;
                }
            }
            if found == 0 {
                println!("(none)");
            }
        }
        Command::Doctor => {
            ensure_initialized(&paths)?;
            ensure_index(&paths, &registry_root)?;
            let reports = checks::run_checks(&paths);
            let (passed, failed) = checks::report_counts(&reports);

            println!("Trellis doctor");
            for report in &reports {
                let mark = if report.ok { "OK" } else { "FAIL" };
                println!("- {:<14} {:<4} {}", report.name, mark, report.detail);
            }
            println!("Summary: {} passed, {} failed", passed, failed);

            checks::summarize(&reports)?;
            println!("Environment is healthy");
        }
    }

    Ok(())
}

fn ensure_initialized(paths: &TrellisPaths) -> Result<()> {
    if !paths.home.exists() {
        bail!("Trellis home not initialized. Run 'trellis init'.");
    }
    Ok(())
}

fn ensure_index(paths: &TrellisPaths, registry_root: &Path) -> Result<()> {
    let index_path = paths.registry.join("index.json");
    if !index_path.exists() {
        sync_registry(paths, registry_root)?;
    }
    Ok(())
}

fn resolve_registry_root(override_root: Option<&Path>) -> Result<PathBuf> {
    match override_root {
        Some(p) => Ok(p.to_path_buf()),
        None => std::env::current_dir()
            .map(|d| d.join("packages"))
            .context("failed to resolve current directory"),
    }
}

fn find_package(
    paths: &TrellisPaths,
    registry_root: &Path,
    name: &str,
) -> Result<crate::registry::index::RegistryEntry> {
    ensure_index(paths, registry_root)?;
    let entries = scan_registry(registry_root)?;
    entries
        .into_iter()
        .find(|e| e.spec.name == name)
        .ok_or_else(|| anyhow!("package '{}' not found in local registry", name))
}
