use anyhow::{bail, Context, Result};
use chrono::Utc;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::core::paths::TrellisPaths;
use crate::core::receipts::{read_receipt, write_receipt, Receipt};
use crate::registry::index::RegistryEntry;
use crate::spec::package::PackageSpec;
use crate::trust::checksum;

pub fn install(paths: &TrellisPaths, entry: &RegistryEntry, spec: &PackageSpec) -> Result<()> {
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

    let spec_dir = entry
        .spec_path
        .parent()
        .context("spec must have parent directory")?;
    let source_root = spec_dir.join(&spec.source.path);
    if !source_root.exists() {
        bail!("source path does not exist: {}", source_root.display());
    }

    if let Some(expected) = &spec.source.checksum_sha256 {
        if source_root.is_file() {
            let actual = checksum::sha256_file(&source_root)?;
            if &actual != expected {
                bail!("checksum mismatch for {}", source_root.display());
            }
        }
    }

    let install_root = paths.cellar.join(&spec.name).join(&spec.version);
    if install_root.exists() {
        fs::remove_dir_all(&install_root)?;
    }
    fs::create_dir_all(&install_root)?;

    let mut installed_files = Vec::new();
    for item in &spec.install.entries {
        let src = source_root.join(item);
        if !src.exists() {
            bail!("install entry missing: {}", src.display());
        }
        let dest = install_root.join(item);
        copy_recursively(&src, &dest, &mut installed_files)?;
    }

    let mut exposed = BTreeMap::new();
    for (name, rel) in &spec.bin {
        let target = install_root.join(rel);
        if !target.exists() {
            bail!("binary target missing: {}", target.display());
        }

        let link_path = paths.bin.join(name);
        if link_path.exists() {
            fs::remove_file(&link_path)
                .or_else(|_| fs::remove_dir_all(&link_path))
                .with_context(|| {
                    format!("failed to clean existing binary {}", link_path.display())
                })?;
        }
        link_or_copy(&target, &link_path)?;
        exposed.insert(name.clone(), target.to_string_lossy().to_string());
    }

    let receipt = Receipt {
        name: spec.name.clone(),
        version: spec.version.clone(),
        installed_at: Utc::now(),
        source_path: source_root.to_string_lossy().to_string(),
        checksum_sha256: spec.source.checksum_sha256.clone(),
        installed_files,
        exposed_binaries: exposed,
        registry: spec.provenance.registry.clone(),
        publisher: spec.provenance.publisher.clone(),
        license: spec.provenance.license.clone(),
    };

    write_receipt(&receipt_path, &receipt)?;
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
