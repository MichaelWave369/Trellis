use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::{Cli, Command};
use crate::core::install;
use crate::core::paths::TrellisPaths;
use crate::core::{remove, state};
use crate::doctor::checks;
use crate::registry::index::{read_index, scan_registry, RegistryEntry};
use crate::registry::sync::sync_registry;
use crate::spec::{load_spec, validate};

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
            let entry = resolve_target(&paths, &registry_root, &pkg)?;
            print_info(&entry.spec);
        }
        Command::Validate { target } => {
            ensure_initialized(&paths)?;
            let entry = resolve_target(&paths, &registry_root, &target)?;
            validate::validate(&entry.spec)?;
            println!(
                "Valid: {} {} ({})",
                entry.spec.name, entry.spec.version, entry.spec.schema_version
            );
        }
        Command::Inspect { target } => {
            ensure_initialized(&paths)?;
            let entry = resolve_target(&paths, &registry_root, &target)?;
            let spec = entry.spec;
            println!("Inspect: {} {}", spec.name, spec.version);
            println!("  Schema: {}", spec.schema_version);
            println!("  Kind: {:?}", spec.kind);
            println!(
                "  Source: {:?} {}",
                spec.source.source_type, spec.source.path
            );
            println!("  Registry: {}", spec.provenance.registry);
            println!("  Publisher: {}", spec.provenance.publisher);
            println!("  License: {}", spec.provenance.license);
            println!("  Dependencies: {}", spec.dependencies.len());
            if !spec.dependencies.is_empty() {
                println!(
                    "  Note: dependencies are declared but not automatically resolved in v0.2"
                );
            }
            println!(
                "  Integrity: checksum={} signature={}",
                spec.source
                    .checksum_sha256
                    .as_ref()
                    .map(|_| "present")
                    .unwrap_or("absent"),
                spec.source.signature.as_deref().unwrap_or("absent")
            );
        }
        Command::Install { pkg, from } => {
            ensure_initialized(&paths)?;
            let entry = match (pkg, from) {
                (Some(name), None) => find_package(&paths, &registry_root, &name)?,
                (None, Some(path)) => load_entry_from_path(&path)?,
                _ => bail!("use exactly one install target: either <pkg> or --from <path>"),
            };
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
                    println!(
                        "- {:<20} {} ({})",
                        receipt.name, receipt.version, receipt.kind
                    );
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

fn print_info(spec: &crate::spec::package::PackageSpec) {
    println!("Package: {}", spec.name);
    println!("Version: {}", spec.version);
    println!("Description: {}", spec.description);
    println!("Homepage: {}", spec.homepage);
    println!("Kind: {:?}", spec.kind);
    println!(
        "Source: {:?} ({})",
        spec.source.source_type, spec.source.path
    );
    println!("Registry: {}", spec.provenance.registry);
    println!("Publisher: {}", spec.provenance.publisher);
    println!("License: {}", spec.provenance.license);
    println!("Dependencies declared: {}", spec.dependencies.len());
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

fn find_package(paths: &TrellisPaths, registry_root: &Path, name: &str) -> Result<RegistryEntry> {
    ensure_index(paths, registry_root)?;
    let entries = scan_registry(registry_root)?;
    entries
        .into_iter()
        .find(|e| e.spec.name == name)
        .ok_or_else(|| anyhow!("package '{}' not found in local registry", name))
}

fn resolve_target(
    paths: &TrellisPaths,
    registry_root: &Path,
    target: &str,
) -> Result<RegistryEntry> {
    let target_path = Path::new(target);
    if target_path.exists() {
        load_entry_from_path(target_path)
    } else {
        find_package(paths, registry_root, target)
    }
}

fn load_entry_from_path(path: &Path) -> Result<RegistryEntry> {
    let spec_path = if path.is_dir() {
        let mut found = None;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry
                .path()
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|name| name.ends_with(".trellis.yaml"))
            {
                found = Some(entry.path());
                break;
            }
        }
        found.ok_or_else(|| anyhow!("no .trellis.yaml spec file found in {}", path.display()))?
    } else {
        path.to_path_buf()
    };

    let spec = load_spec(&spec_path)?;
    Ok(RegistryEntry { spec_path, spec })
}
