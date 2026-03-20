use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::ui;
use crate::cli::{Cli, Command};
use crate::core::install;
use crate::core::lock::{self, LockedPackage};
use crate::core::paths::TrellisPaths;
use crate::core::receipts::read_receipt;
use crate::core::scaffold::{self, ScaffoldKind};
use crate::core::{remove, state};
use crate::doctor::checks;
use crate::registry::index::{read_index, RegistryEntry};
use crate::registry::sync::sync_registry;
use crate::spec::{load_spec, validate};

pub fn run(cli: Cli) -> Result<()> {
    let paths = TrellisPaths::resolve(cli.home.as_deref())?;
    let registry_root = resolve_registry_root(cli.registry_root.as_deref())?;
    let profile = cli.profile.clone();

    match cli.command {
        Command::Init => {
            ui::header("Initialize Trellis");
            ui::step(format!("Creating state at {}", paths.home.display()));
            state::init(&paths)?;
            ui::ok("Trellis home initialized");
            ui::info("Next step: run `trellis update` to materialize the registry index");
        }
        Command::Update => {
            ensure_initialized(&paths)?;
            ui::header("Registry Update");
            ui::step("Refreshing enabled registry sources");
            let report = sync_registry(&paths, Some(&registry_root))?;
            ui::ok(format!(
                "Index refreshed: {} package(s), {} malformed",
                report.index.packages.len(),
                report.index.skipped.len()
            ));
            println!("\nRegistry summary");
            println!(
                "{:<16} {:>8} {:>8}  {:<30}",
                "Name", "Pkgs", "Skipped", "Refreshed"
            );
            for registry in report.index.registries {
                println!(
                    "{:<16} {:>8} {:>8}  {:<30}",
                    registry.name,
                    registry.package_count,
                    registry.skipped_count,
                    registry.refreshed_at
                );
            }
        }
        Command::Search { query } => {
            ensure_initialized(&paths)?;
            ensure_index(&paths, &registry_root)?;
            let query_lower = query.to_lowercase();
            let index = read_index(&paths.registry_index)?;

            ui::header("Search Packages");
            ui::info(format!("Query: '{}'", query));
            println!(
                "{:<20} {:<10} {:<8} {:<14} Description",
                "Name", "Version", "Kind", "Registry"
            );

            let mut matches = index
                .packages
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&query_lower)
                        || p.description.to_lowercase().contains(&query_lower)
                })
                .collect::<Vec<_>>();
            matches.sort_by(|a, b| {
                b.featured
                    .cmp(&a.featured)
                    .then_with(|| a.name.cmp(&b.name))
            });

            let mut matched = 0usize;
            for pkg in matches {
                let featured = if pkg.featured { "featured" } else { "catalog" };
                println!(
                    "{:<20} {:<10} {:<8} {:<14} {} [{}]",
                    pkg.name, pkg.version, pkg.kind, pkg.registry, pkg.description, featured
                );
                matched += 1;
            }

            if matched == 0 {
                ui::warn("No packages matched your query");
                ui::info("Try a broader query, or run `trellis update` to refresh index state");
            } else {
                ui::ok(format!("{} result(s)", matched));
            }
        }
        Command::Info { pkg } => {
            ensure_initialized(&paths)?;
            ui::header("Package Info");
            let entry = resolve_target(&paths, &registry_root, &pkg)?;
            print_info(&entry.spec, Some(&entry.registry));
        }
        Command::Validate { target } => {
            ensure_initialized(&paths)?;
            ui::header("Validate Package Spec");
            let entry = resolve_target(&paths, &registry_root, &target)?;
            validate::validate(&entry.spec)?;
            ui::ok(format!(
                "Valid: {} {} (schema {})",
                entry.spec.name, entry.spec.version, entry.spec.schema_version
            ));
        }
        Command::Inspect { target } => {
            ensure_initialized(&paths)?;
            ui::header("Inspect Package Spec");
            let entry = resolve_target(&paths, &registry_root, &target)?;
            let spec = entry.spec;
            println!("Package       : {} {}", spec.name, spec.version);
            println!("Schema        : {}", spec.schema_version);
            println!("Kind          : {:?}", spec.kind);
            println!(
                "Source        : {:?} {}",
                spec.source.source_type, spec.source.path
            );
            println!("Registry      : {}", spec.provenance.registry);
            println!("Publisher     : {}", spec.provenance.publisher);
            println!("License       : {}", spec.provenance.license);
            println!("Dependencies  : {}", spec.dependencies.len());
            if !spec.dependencies.is_empty() {
                ui::warn("Dependencies are declared; automatic dependency resolution is deferred");
            }
            println!(
                "Integrity     : checksum={} signature={}",
                spec.source
                    .checksum_sha256
                    .as_ref()
                    .map(|_| "present")
                    .unwrap_or("absent"),
                spec.source.signature.as_deref().unwrap_or("absent")
            );
        }
        Command::Receipt { pkg } => {
            ensure_initialized(&paths)?;
            render_receipt(&paths, &pkg)?;
        }
        Command::Install { pkg, from } => {
            ensure_initialized(&paths)?;
            ui::header("Install Package");
            ui::step("Resolving package target");
            let lock_profile = profile.as_str();
            let mut installed_for_lock = Vec::new();

            match (pkg, from) {
                (Some(name), None) => {
                    let order = resolve_install_order(&paths, &registry_root, &name)?;
                    println!("Resolution order: {}", order.iter().map(|e| e.spec.name.as_str()).collect::<Vec<_>>().join(" -> "));
                    for entry in order {
                        if is_installed(&paths, &entry.spec.name) {
                            ui::info(format!("Skipping already installed dependency: {}", entry.spec.name));
                            installed_for_lock.push(LockedPackage { name: entry.spec.name.clone(), version: entry.spec.version.clone(), registry: entry.registry.clone()});
                            continue;
                        }
                        ui::step(format!("Installing {} {}", entry.spec.name, entry.spec.version));
                        install::install(&paths, &entry, &entry.spec)?;
                        installed_for_lock.push(LockedPackage { name: entry.spec.name.clone(), version: entry.spec.version.clone(), registry: entry.registry.clone()});
                    }
                    lock::write_lock(&paths, lock_profile, installed_for_lock.clone())?;
                    ui::ok(format!("Install complete (profile: {})", lock_profile));
                }
                (None, Some(path)) => {
                    let entry = load_entry_from_path(&path)?;
                    ui::step("Applying install plan");
                    install::install(&paths, &entry, &entry.spec)?;
                    installed_for_lock.push(LockedPackage { name: entry.spec.name.clone(), version: entry.spec.version.clone(), registry: entry.registry.clone()});
                    lock::write_lock(&paths, lock_profile, installed_for_lock.clone())?;
                    ui::ok(format!("Installed {} {} from local path", entry.spec.name, entry.spec.version));
                }
                _ => bail!("use exactly one install target: either <pkg> or --from <path>"),
            }

            ui::info(format!("Lock state written: {}", lock::lock_path(&paths, lock_profile).display()));
            ui::info("Current dependency model resolves direct dependencies and nested declarations deterministically; complex conflict solving is deferred.");
        }
        Command::Remove { pkg } => {
            ensure_initialized(&paths)?;
            ui::header("Remove Package");
            ui::step(format!("Removing {}", pkg));
            remove::remove(&paths, &pkg)?;
            ui::ok(format!("Removed {}", pkg));
        }
        Command::List => {
            ensure_initialized(&paths)?;
            ui::header("Installed Packages");
            println!(
                "{:<20} {:<10} {:<10} {:<12} {:<10}",
                "Name", "Version", "Kind", "Registry", "Trust"
            );
            let mut found = 0usize;
            for entry in fs::read_dir(&paths.receipts)? {
                let entry = entry?;
                if entry.path().extension().and_then(|v| v.to_str()) == Some("json") {
                    let receipt = read_receipt(&entry.path())?;
                    println!(
                        "{:<20} {:<10} {:<10} {:<12} {:<10}",
                        receipt.name,
                        receipt.version,
                        receipt.kind,
                        receipt.registry.name,
                        format!("{:?}", receipt.trust.checksum_state).to_lowercase()
                    );
                    found += 1;
                }
            }
            if found == 0 {
                ui::warn("No installed packages");
                ui::info("Install a package with `trellis install <pkg>`");
            } else {
                ui::ok(format!("{} installed package(s)", found));
            }
        }
        Command::Scaffold {
            package_name,
            kind,
            out,
        } => {
            ui::header("Scaffold Package");
            crate::spec::validate::validate_name(&package_name)?;
            let kind = ScaffoldKind::from_str(&kind)?;
            let root = out.unwrap_or_else(|| PathBuf::from("packages"));
            ui::step(format!("Creating scaffold in {}", root.display()));
            let package_dir = scaffold::scaffold_package(&root, &package_name, kind)?;
            ui::ok(format!("Scaffold created at {}", package_dir.display()));
            ui::info(format!(
                "Next: trellis validate {}",
                package_dir
                    .join(format!("{}.trellis.yaml", package_name))
                    .display()
            ));
        }
        Command::Readiness { target } => {
            ui::header("Submission Readiness");
            if !Path::new(&target).exists() {
                ensure_initialized(&paths)?;
            }
            let entry = resolve_target(&paths, &registry_root, &target)?;
            validate::validate(&entry.spec)?;
            println!("Checklist");
            println!("  [ok] spec validates");
            println!(
                "  [{}] provenance.publisher set",
                if entry.spec.provenance.publisher.starts_with("TODO") {
                    "warn"
                } else {
                    "ok"
                }
            );
            println!(
                "  [{}] provenance.license set",
                if entry.spec.provenance.license.starts_with("TODO") {
                    "warn"
                } else {
                    "ok"
                }
            );
            println!(
                "  [{}] checksum declared",
                if entry.spec.source.checksum_sha256.is_some() {
                    "ok"
                } else {
                    "warn"
                }
            );
            println!(
                "  [{}] signature metadata",
                match crate::trust::assess_signature(entry.spec.source.signature.as_deref()).state {
                    crate::trust::SignatureState::Present => "ok",
                    crate::trust::SignatureState::Missing => "warn",
                    crate::trust::SignatureState::Malformed => "warn",
                    crate::trust::SignatureState::Unsupported => "warn",
                }
            );
            println!(
                "  [ok] install entries: {}",
                entry.spec.install.entries.len()
            );
            println!("  [ok] bin mappings: {}", entry.spec.bin.len());
            ui::info("For official registry submissions, include package folder, payload, and spec in one PR.");
        }
        Command::Seed | Command::Bootstrap => {
            run_seed(&paths, &registry_root)?;
        }
        Command::Verify => {
            ensure_initialized(&paths)?;
            ui::header("Verify Installed State");
            let issues = verify_state(&paths)?;
            if issues.is_empty() {
                ui::ok("Installed state matches receipts and lock metadata");
            } else {
                for issue in &issues { ui::warn(issue); }
                bail!("verify found {} issue(s)", issues.len());
            }
        }
        Command::Repair => {
            ensure_initialized(&paths)?;
            ui::header("Repair Installed State");
            let repairs = repair_state(&paths)?;
            for line in &repairs { ui::info(line); }
            let issues = verify_state(&paths)?;
            if issues.is_empty() {
                ui::ok("Repair completed; state is now consistent");
            } else {
                for issue in &issues { ui::warn(issue); }
                bail!("repair could not resolve all issues");
            }
        }
        Command::Doctor => {
            ensure_initialized(&paths)?;
            ensure_index(&paths, &registry_root)?;
            ui::header("Trellis Doctor");
            ui::step("Running health and trust checks");

            let reports = checks::run_checks(&paths);
            let (passed, warnings, failed) = checks::report_counts(&reports);

            println!("\nChecks");
            println!("{:<16} {:<6} {:<60}", "Check", "State", "Detail");
            for report in &reports {
                println!(
                    "{:<16} {:<6} {:<60}",
                    report.name,
                    ui::doctor_mark(report.status),
                    report.detail
                );
                if let Some(remediation) = &report.remediation {
                    println!("  remediation: {}", remediation);
                }
            }

            println!("\nSummary");
            println!("  Passed   : {}", passed);
            println!("  Warnings : {}", warnings);
            println!("  Failed   : {}", failed);

            checks::summarize(&reports)?;
            ui::ok("Environment is healthy enough for local operation");
        }
    }

    Ok(())
}

fn render_receipt(paths: &TrellisPaths, pkg: &str) -> Result<()> {
    ui::header("Installed Receipt");
    let receipt_path = paths.receipts.join(format!("{}.json", pkg));
    if !receipt_path.exists() {
        bail!(
            "receipt for '{}' not found. Install the package first with `trellis install {}`",
            pkg,
            pkg
        );
    }

    let receipt = read_receipt(&receipt_path)?;
    println!("Package       : {} {}", receipt.name, receipt.version);
    println!("Installed at  : {}", receipt.installed_at);
    println!("Transaction   : {}", receipt.transaction_id);
    println!(
        "Registry      : {} ({})",
        receipt.registry.name, receipt.registry.source_path
    );
    println!(
        "Provenance    : publisher={} license={} declared_registry={}",
        receipt.provenance.publisher,
        receipt.provenance.license,
        receipt.provenance.declared_registry
    );
    println!(
        "Trust         : checksum={:?} signature={:?}",
        receipt.trust.checksum_state, receipt.provenance.signature.state
    );
    println!(
        "Platform      : os={} arch={} matched={}",
        receipt.platform_evaluated.os,
        receipt.platform_evaluated.arch,
        receipt.platform_evaluated.matched
    );
    println!("Dependencies  : {}", receipt.dependencies_declared.len());

    println!("Exposed bins  :");
    if receipt.exposed_binaries.is_empty() {
        println!("  - none");
    } else {
        for (name, path) in &receipt.exposed_binaries {
            println!("  - {} -> {}", name, path);
        }
    }

    println!("Post-install  :");
    if receipt.post_install_actions.is_empty() {
        println!("  - none");
    } else {
        for action in &receipt.post_install_actions {
            println!("  - {}", action);
        }
    }

    println!("Warnings      :");
    if receipt.trust.warnings.is_empty() {
        println!("  - none");
    } else {
        for warning in &receipt.trust.warnings {
            println!("  - {}", warning);
        }
    }

    println!("Installed files: {}", receipt.installed_files.len());
    Ok(())
}

fn run_seed(paths: &TrellisPaths, registry_root: &Path) -> Result<()> {
    ui::header("Trellis Seed");
    ui::step("Ensuring local Trellis state exists");
    state::init(paths)?;
    ui::ok(format!("State ready at {}", paths.home.display()));

    ui::step("Refreshing official registry metadata");
    let report = sync_registry(paths, Some(registry_root))?;
    ui::ok(format!(
        "Registry ready: {} package(s), {} malformed",
        report.index.packages.len(),
        report.index.skipped.len()
    ));

    ui::step("Running health and trust checks");
    let reports = checks::run_checks(paths);
    let (passed, warnings, failed) = checks::report_counts(&reports);
    println!(
        "Health summary: {} passed, {} warning(s), {} failed",
        passed, warnings, failed
    );
    for report in reports
        .iter()
        .filter(|r| r.status == checks::CheckStatus::Fail)
    {
        println!("  - {}: {}", report.name, report.detail);
        if let Some(remediation) = &report.remediation {
            println!("    remediation: {}", remediation);
        }
    }
    checks::summarize(&reports)?;

    let index = read_index(&paths.registry_index)?;
    let mut featured = index
        .packages
        .iter()
        .filter(|p| p.featured)
        .map(|p| p.name.clone())
        .collect::<Vec<_>>();
    featured.sort();
    featured.dedup();

    println!(
        "
Featured packages"
    );
    if featured.is_empty() {
        println!("  - (none declared)");
    } else {
        for name in &featured {
            println!("  - {}", name);
        }
    }

    let recommended = "vineyard-core";
    println!(
        "
Recommended first package: {}",
        recommended
    );
    println!(
        "Why: establishes core substrate commands and environment/path visibility for Trellis operators."
    );

    if !is_installed(paths, recommended) {
        ui::info(format!(
            "Install now: trellis --home {} --registry-root {} install {}",
            paths.home.display(),
            registry_root.display(),
            recommended
        ));
    } else {
        ui::ok(format!("{} is already installed", recommended));
    }

    println!(
        "
Paths"
    );
    println!("  Trellis home : {}", paths.home.display());
    println!("  Registry idx : {}", paths.registry_index.display());
    println!("  Bin dir      : {}", paths.bin.display());
    println!("  Receipts dir : {}", paths.receipts.display());
    println!(
        "  PATH hint    : add '{}' to PATH to run installed binaries directly",
        paths.bin.display()
    );

    println!(
        "
Try next"
    );
    println!(
        "  trellis --home {} --registry-root {} search cli",
        paths.home.display(),
        registry_root.display()
    );
    println!(
        "  trellis --home {} --registry-root {} install overstrings-cli",
        paths.home.display(),
        registry_root.display()
    );
    println!(
        "  trellis --home {} receipt vineyard-core",
        paths.home.display()
    );

    ui::ok("Seed flow complete");
    Ok(())
}

fn is_installed(paths: &TrellisPaths, pkg: &str) -> bool {
    paths.receipts.join(format!("{}.json", pkg)).exists()
}

fn resolve_install_order(paths: &TrellisPaths, registry_root: &Path, root: &str) -> Result<Vec<RegistryEntry>> {
    ensure_index(paths, registry_root)?;
    let index = read_index(&paths.registry_index)?;

    let mut by_name = std::collections::BTreeMap::new();
    for pkg in index.packages {
        by_name.entry(pkg.name.clone()).or_insert(pkg);
    }

    let mut order = Vec::new();
    let mut visiting = std::collections::BTreeSet::new();
    let mut visited = std::collections::BTreeSet::new();

    fn visit(
        name: &str,
        by_name: &std::collections::BTreeMap<String, crate::registry::index::IndexedPackage>,
        visiting: &mut std::collections::BTreeSet<String>,
        visited: &mut std::collections::BTreeSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if visited.contains(name) { return Ok(()); }
        if !visiting.insert(name.to_string()) { bail!("dependency cycle detected at '{}'", name); }
        let pkg = by_name.get(name).ok_or_else(|| anyhow!("missing dependency '{}' in active registry", name))?;
        let mut deps = pkg.dependencies.clone();
        deps.sort();
        for dep in deps {
            if let Some(other) = by_name.get(&dep) {
                if other.version != pkg.version && dep == pkg.name {
                    bail!("simple dependency conflict detected for '{}': multiple versions", dep);
                }
            }
            visit(&dep, by_name, visiting, visited, order)?;
        }
        visiting.remove(name);
        visited.insert(name.to_string());
        order.push(name.to_string());
        Ok(())
    }

    visit(root, &by_name, &mut visiting, &mut visited, &mut order)?;

    let mut entries = Vec::new();
    for name in order {
        let pkg = by_name.get(&name).ok_or_else(|| anyhow!("resolved package '{}' missing", name))?;
        let spec_path = PathBuf::from(&pkg.spec_path);
        let spec = load_spec(&spec_path)?;
        entries.push(RegistryEntry { registry: pkg.registry.clone(), spec_path, spec_rel_path: pkg.spec_rel_path.clone(), spec });
    }

    Ok(entries)
}

fn verify_state(paths: &TrellisPaths) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    for entry in fs::read_dir(&paths.receipts)? {
        let entry = entry?;
        if entry.path().extension().and_then(|v| v.to_str()) != Some("json") { continue; }
        let receipt = read_receipt(&entry.path())?;
        let install_root = paths.cellar.join(&receipt.name).join(&receipt.version);
        if !install_root.exists() {
            issues.push(format!("missing install root for {}: {}", receipt.name, install_root.display()));
        }
        for (bin, target) in &receipt.exposed_binaries {
            let bin_path = paths.bin.join(bin);
            if !bin_path.exists() {
                issues.push(format!("missing exposed binary {}", bin));
            }
            if !Path::new(target).exists() {
                issues.push(format!("missing binary target for {} -> {}", bin, target));
            }
        }
    }

    if let Ok(lock) = lock::read_lock(paths, "default") {
        for locked in lock.packages {
            if !is_installed(paths, &locked.name) {
                issues.push(format!("lock references {} but receipt is missing", locked.name));
            }
        }
    }

    Ok(issues)
}

fn repair_state(paths: &TrellisPaths) -> Result<Vec<String>> {
    let mut actions = Vec::new();
    for entry in fs::read_dir(&paths.receipts)? {
        let entry = entry?;
        if entry.path().extension().and_then(|v| v.to_str()) != Some("json") { continue; }
        let receipt = read_receipt(&entry.path())?;
        for (bin, target) in &receipt.exposed_binaries {
            let bin_path = paths.bin.join(bin);
            if bin_path.exists() { continue; }
            if !Path::new(target).exists() { continue; }
            #[cfg(unix)]
            {
                if std::os::unix::fs::symlink(target, &bin_path).is_ok() {
                    actions.push(format!("repaired symlink {} -> {}", bin_path.display(), target));
                    continue;
                }
            }
            fs::copy(target, &bin_path)?;
            actions.push(format!("repaired copy {} -> {}", bin_path.display(), target));
        }
    }
    Ok(actions)
}

fn print_info(spec: &crate::spec::package::PackageSpec, resolved_registry: Option<&str>) {
    println!("Name          : {}", spec.name);
    println!("Version       : {}", spec.version);
    println!("Description   : {}", spec.description);
    println!("Homepage      : {}", spec.homepage);
    println!("Kind          : {:?}", spec.kind);
    println!(
        "Source        : {:?} ({})",
        spec.source.source_type, spec.source.path
    );
    if let Some(registry) = resolved_registry {
        println!("Registry      : {}", registry);
    }
    println!("Publisher     : {}", spec.provenance.publisher);
    println!("License       : {}", spec.provenance.license);
    println!("Dependencies  : {}", spec.dependencies.len());
    if let Some(platform) = &spec.platform {
        println!("Platform.os   : {:?}", platform.os);
        println!("Platform.arch : {:?}", platform.arch);
    }
    let checksum_status = if spec.source.checksum_sha256.is_some() {
        "declared (verified during install)"
    } else {
        "unavailable"
    };
    let signature_assessment = crate::trust::assess_signature(spec.source.signature.as_deref());
    println!("Checksum      : {}", checksum_status);
    println!(
        "Signature     : {:?} ({})",
        signature_assessment.state, signature_assessment.note
    );
}

fn ensure_initialized(paths: &TrellisPaths) -> Result<()> {
    if !paths.home.exists() {
        bail!("Trellis home not initialized. Run `trellis init` first, then `trellis update`.");
    }
    Ok(())
}

fn ensure_index(paths: &TrellisPaths, registry_root: &Path) -> Result<()> {
    if !paths.registry_index.exists() {
        ui::warn("Registry index is missing; running implicit `trellis update`");
        sync_registry(paths, Some(registry_root))?;
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
    let index = read_index(&paths.registry_index)?;

    let pkg = index
        .packages
        .into_iter()
        .find(|pkg| pkg.name == name)
        .ok_or_else(|| {
            anyhow!(
                "package '{}' not found in active registries. Run `trellis search {}` to discover available packages",
                name,
                name
            )
        })?;

    let spec_path = PathBuf::from(pkg.spec_path);
    let spec = load_spec(&spec_path)?;
    Ok(RegistryEntry {
        registry: pkg.registry,
        spec_path,
        spec_rel_path: pkg.spec_rel_path,
        spec,
    })
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
    Ok(RegistryEntry {
        registry: spec.provenance.registry.clone(),
        spec_rel_path: spec_path.to_string_lossy().to_string(),
        spec_path,
        spec,
    })
}
