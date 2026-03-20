use anyhow::{bail, Context, Result};
use chrono::Utc;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::core::paths::TrellisPaths;
use crate::core::receipts::{
    read_receipt, write_receipt, PlatformEvaluation, ProvenanceReceipt, Receipt, RegistryReceipt,
    SourceReceipt, TrustSummary,
};
use crate::registry::index::RegistryEntry;
use crate::spec::package::{PackageKind, PackageSpec, SourceType};
use crate::spec::validate::platform_matches;
use crate::trust::{assess_signature, checksum, ChecksumState};

pub fn install(paths: &TrellisPaths, entry: &RegistryEntry, spec: &PackageSpec) -> Result<()> {
    if !platform_matches(spec) {
        bail!(
            "package '{}' {} does not support this platform (os={}, arch={})",
            spec.name,
            spec.version,
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }

    let receipt_path = paths.receipts.join(format!("{}.json", spec.name));
    if receipt_path.exists() {
        let existing = read_receipt(&receipt_path)?;
        bail!(
            "package '{}' {} is already installed (installed version: {}). Run `trellis remove {}` first",
            spec.name,
            spec.version,
            existing.version,
            spec.name
        );
    }

    let install_root = paths.cellar.join(&spec.name).join(&spec.version);
    if install_root.exists() {
        bail!(
            "install target already exists at {}. Remove existing state before reinstalling",
            install_root.display()
        );
    }

    let spec_dir = entry
        .spec_path
        .parent()
        .context("spec must have parent directory")?;
    let source_root = spec_dir.join(&spec.source.path);
    validate_source_shape(&source_root, &spec.source.source_type)?;

    for bin_name in spec.bin.keys() {
        if let Some(owner) = find_binary_owner(paths, bin_name)? {
            bail!(
                "binary '{}' is already managed by '{}'. Remove '{}' first",
                bin_name,
                owner,
                owner
            );
        }

        let link_path = paths.bin.join(bin_name);
        if link_path.exists() {
            bail!(
                "binary '{}' already exists at {}. Refusing to overwrite unmanaged binary",
                bin_name,
                link_path.display()
            );
        }
    }

    let (checksum_state, checksum_actual) = verify_checksum(spec, &source_root)?;
    if checksum_state == ChecksumState::Mismatched {
        bail!("checksum mismatch for {}", source_root.display());
    }

    let signature = assess_signature(spec.source.signature.as_deref());

    if !spec.dependencies.is_empty() {
        println!(
            "Dependency declarations: {} (auto-resolution remains deferred)",
            spec.dependencies.len()
        );
    }

    fs::create_dir_all(&install_root)?;

    let mut installed_files = Vec::new();
    for item in &spec.install.entries {
        let src = source_root.join(item);
        if !src.exists() {
            bail!("install entry missing: {}", src.display());
        }
        let dest = install_root.join(item);
        if dest.exists() {
            bail!(
                "install target collision at {}. Refusing to overwrite existing managed files",
                dest.display()
            );
        }
        copy_recursively(&src, &dest, &mut installed_files)?;
    }

    let mut exposed = BTreeMap::new();
    for (name, rel) in &spec.bin {
        let target = install_root.join(rel);
        if !target.exists() {
            bail!("binary target missing: {}", target.display());
        }

        let link_path = paths.bin.join(name);
        link_or_copy(&target, &link_path)?;
        exposed.insert(name.clone(), target.to_string_lossy().to_string());
    }

    installed_files.sort();
    let post_install_actions = spec
        .post_install
        .as_ref()
        .map(|p| vec![format!("declared:{}:{}", p.policy, p.command)])
        .unwrap_or_default();

    let mut warnings = Vec::new();
    if checksum_state != ChecksumState::Verified {
        warnings.push(format!(
            "checksum state is {:?}; verify source integrity policy before production use",
            checksum_state
        ));
    }
    if signature.state != crate::trust::SignatureState::Present {
        warnings.push(signature.note.clone());
    }

    let trust_summary = TrustSummary {
        checksum_state: checksum_state.clone(),
        signature_state: signature.state.clone(),
        warnings: warnings.clone(),
        summary: format!(
            "checksum={:?}; signature={:?}",
            checksum_state, signature.state
        ),
    };

    let (constraints_os, constraints_arch) = if let Some(platform) = &spec.platform {
        (platform.os.clone(), platform.arch.clone())
    } else {
        (Vec::new(), Vec::new())
    };

    let receipt = Receipt {
        schema_version: "0.4".to_string(),
        transaction_id: format!("install-{}-{}", spec.name, Utc::now().timestamp_millis()),
        name: spec.name.clone(),
        version: spec.version.clone(),
        kind: match spec.kind {
            PackageKind::Binary => "binary".to_string(),
            PackageKind::Source => "source".to_string(),
        },
        installed_at: Utc::now(),
        registry: RegistryReceipt {
            name: entry.registry.clone(),
            source_path: entry.spec_path.to_string_lossy().to_string(),
        },
        source: SourceReceipt {
            source_type: source_type_label(&spec.source.source_type).to_string(),
            source_path: source_root.to_string_lossy().to_string(),
            checksum_expected_sha256: spec.source.checksum_sha256.clone(),
            checksum_actual_sha256: checksum_actual,
        },
        provenance: ProvenanceReceipt {
            publisher: spec.provenance.publisher.clone(),
            license: spec.provenance.license.clone(),
            declared_registry: spec.provenance.registry.clone(),
            signature,
        },
        dependencies_declared: spec.dependencies.clone(),
        platform_evaluated: PlatformEvaluation {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            matched: true,
            constraints_os,
            constraints_arch,
        },
        installed_files,
        exposed_binaries: exposed.clone(),
        post_install_actions,
        trust: trust_summary,
    };

    write_receipt(&receipt_path, &receipt)?;

    print_install_report(
        spec,
        entry,
        &source_root,
        &checksum_state,
        &receipt,
        &warnings,
    );
    Ok(())
}

fn verify_checksum(
    spec: &PackageSpec,
    source_root: &Path,
) -> Result<(ChecksumState, Option<String>)> {
    let Some(expected) = &spec.source.checksum_sha256 else {
        return Ok((ChecksumState::Unavailable, None));
    };

    let actual = match spec.source.source_type {
        SourceType::File | SourceType::Archive => checksum::sha256_file(source_root)?,
        SourceType::Dir => checksum::sha256_dir(source_root)?,
    };

    if &actual == expected {
        Ok((ChecksumState::Verified, Some(actual)))
    } else {
        Ok((ChecksumState::Mismatched, Some(actual)))
    }
}

fn find_binary_owner(paths: &TrellisPaths, bin_name: &str) -> Result<Option<String>> {
    for entry in fs::read_dir(&paths.receipts)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }

        let receipt = read_receipt(&path)?;
        if receipt.exposed_binaries.contains_key(bin_name) {
            return Ok(Some(receipt.name));
        }
    }
    Ok(None)
}

fn source_type_label(source_type: &SourceType) -> &'static str {
    match source_type {
        SourceType::File => "local_file",
        SourceType::Dir => "local_dir",
        SourceType::Archive => "local_archive",
    }
}

fn print_install_report(
    spec: &PackageSpec,
    entry: &RegistryEntry,
    source_root: &Path,
    checksum_state: &ChecksumState,
    receipt: &Receipt,
    warnings: &[String],
) {
    println!("Install summary");
    println!("- Package: {} {}", spec.name, spec.version);
    println!("- Registry: {}", entry.registry);
    println!(
        "- Source: {} ({})",
        source_type_label(&spec.source.source_type),
        source_root.display()
    );
    println!("- Checksum: {:?}", checksum_state);
    println!("- Signature: {:?}", receipt.provenance.signature.state);
    println!("- Publisher: {}", spec.provenance.publisher);
    println!("- Dependencies declared: {}", spec.dependencies.len());
    if receipt.exposed_binaries.is_empty() {
        println!("- Exposed binaries: none");
    } else {
        println!("- Exposed binaries:");
        for (name, target) in &receipt.exposed_binaries {
            println!("  - {} -> {}", name, target);
        }
    }

    if warnings.is_empty() {
        println!("- Trust warnings: none");
    } else {
        println!("- Trust warnings:");
        for warning in warnings {
            println!("  - {}", warning);
        }
    }
}

fn validate_source_shape(path: &Path, source_type: &SourceType) -> Result<()> {
    match source_type {
        SourceType::File => {
            if !path.is_file() {
                bail!("source.type local_file requires a file: {}", path.display());
            }
        }
        SourceType::Dir => {
            if !path.is_dir() {
                bail!(
                    "source.type local_dir requires a directory: {}",
                    path.display()
                );
            }
        }
        SourceType::Archive => {
            if !path.is_file() {
                bail!(
                    "source.type local_archive requires a file: {}",
                    path.display()
                );
            }
            let name = path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
            if !(name.ends_with(".tar") || name.ends_with(".tar.gz") || name.ends_with(".zip")) {
                bail!("local_archive path must end with .tar, .tar.gz, or .zip");
            }
        }
    }
    Ok(())
}

fn copy_recursively(src: &Path, dest: &Path, installed_files: &mut Vec<String>) -> Result<()> {
    if src.is_file() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dest)?;
        installed_files.push(dest.to_string_lossy().to_string());
        return Ok(());
    }

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
            installed_files.push(target.to_string_lossy().to_string());
        }
    }
    Ok(())
}

fn link_or_copy(target: &Path, link_path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        if std::os::unix::fs::symlink(target, link_path).is_ok() {
            return Ok(());
        }
    }

    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_file(target, link_path).is_ok() {
            return Ok(());
        }
    }

    fs::copy(target, link_path)
        .with_context(|| format!("failed to copy binary {}", link_path.display()))?;
    Ok(())
}
