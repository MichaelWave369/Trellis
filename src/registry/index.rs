use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::spec::{
    load_spec,
    package::{PackageSpec, SourceType},
    validate,
};

#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub registry: String,
    pub spec_path: PathBuf,
    pub spec_rel_path: String,
    pub spec: PackageSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: String,
    pub registry: String,
    pub spec_path: String,
    pub spec_rel_path: String,
    pub source_type: String,
    pub dependencies: Vec<String>,
    pub platform_os: Vec<String>,
    pub platform_arch: Vec<String>,
    pub provenance: IndexedProvenance,
    pub integrity: IndexedIntegrity,
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedProvenance {
    pub publisher: String,
    pub license: String,
    pub registry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedIntegrity {
    pub checksum_sha256: Option<String>,
    pub signature: Option<String>,
    pub checksum_declared: bool,
    pub signature_declared: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySummary {
    pub name: String,
    pub source_path: String,
    pub package_count: usize,
    pub skipped_count: usize,
    pub revision: Option<String>,
    pub refreshed_at: DateTime<Utc>,
    pub mirrors: Vec<MirrorSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorSummary {
    pub name: String,
    pub location: String,
    pub priority: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedSpec {
    pub registry: String,
    pub spec_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIndex {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub registries: Vec<RegistrySummary>,
    pub packages: Vec<IndexedPackage>,
    pub skipped: Vec<SkippedSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    pub schema_version: String,
    pub name: String,
    pub title: String,
    pub revision: Option<String>,
    pub generated_at: Option<DateTime<Utc>>,
    pub featured_packages: Option<Vec<String>>,
    pub provenance: RegistryMetadataProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadataProvenance {
    pub maintainer: String,
    pub trust_policy: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub entries: Vec<RegistryEntry>,
    pub skipped: Vec<SkippedSpec>,
    pub metadata: RegistryMetadata,
}

pub fn scan_registry(registry_name: &str, registry_root: &Path) -> ScanReport {
    let metadata = read_registry_metadata(registry_name, registry_root)
        .unwrap_or_else(|_| default_registry_metadata(registry_name, registry_root));

    let mut entries = Vec::new();
    let mut skipped = Vec::new();
    let mut spec_files = Vec::new();

    for entry in WalkDir::new(registry_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path
            .file_name()
            .and_then(|v| v.to_str())
            .is_some_and(|name| name.ends_with(".trellis.yaml"))
        {
            spec_files.push(path.to_path_buf());
        }
    }

    spec_files.sort();

    for spec_path in spec_files {
        match load_spec(&spec_path).and_then(|spec| {
            validate::validate(&spec)?;
            Ok(spec)
        }) {
            Ok(spec) => {
                let rel = spec_path
                    .strip_prefix(registry_root)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| spec_path.to_string_lossy().to_string());
                entries.push(RegistryEntry {
                    registry: registry_name.to_string(),
                    spec_path: spec_path.clone(),
                    spec_rel_path: rel,
                    spec,
                });
            }
            Err(err) => skipped.push(SkippedSpec {
                registry: registry_name.to_string(),
                spec_path: spec_path.to_string_lossy().to_string(),
                reason: err.to_string(),
            }),
        }
    }

    entries.sort_by(|a, b| {
        a.spec
            .name
            .cmp(&b.spec.name)
            .then_with(|| a.spec.version.cmp(&b.spec.version))
            .then_with(|| a.spec_rel_path.cmp(&b.spec_rel_path))
    });

    ScanReport {
        entries,
        skipped,
        metadata,
    }
}

pub fn write_index(index: &RegistryIndex, index_path: &Path) -> Result<()> {
    let serialized = serde_json::to_string_pretty(index)?;
    fs::write(index_path, serialized)
        .with_context(|| format!("failed to write index {}", index_path.display()))
}

pub fn read_index(index_path: &Path) -> Result<RegistryIndex> {
    let contents = fs::read_to_string(index_path)
        .with_context(|| format!("failed to read index {}", index_path.display()))?;
    let index = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse index {}", index_path.display()))?;
    Ok(index)
}

pub fn build_package(entry: &RegistryEntry, featured_packages: &[String]) -> IndexedPackage {
    let source_type = match entry.spec.source.source_type {
        SourceType::File => "local_file",
        SourceType::Dir => "local_dir",
        SourceType::Archive => "local_archive",
    }
    .to_string();

    let (platform_os, platform_arch) = if let Some(p) = &entry.spec.platform {
        (p.os.clone(), p.arch.clone())
    } else {
        (Vec::new(), Vec::new())
    };

    IndexedPackage {
        name: entry.spec.name.clone(),
        version: entry.spec.version.clone(),
        description: entry.spec.description.clone(),
        kind: format!("{:?}", entry.spec.kind).to_lowercase(),
        registry: entry.registry.clone(),
        spec_path: entry.spec_path.to_string_lossy().to_string(),
        spec_rel_path: entry.spec_rel_path.clone(),
        source_type,
        dependencies: entry.spec.dependencies.clone(),
        platform_os,
        platform_arch,
        provenance: IndexedProvenance {
            publisher: entry.spec.provenance.publisher.clone(),
            license: entry.spec.provenance.license.clone(),
            registry: entry.spec.provenance.registry.clone(),
        },
        integrity: IndexedIntegrity {
            checksum_sha256: entry.spec.source.checksum_sha256.clone(),
            signature: entry.spec.source.signature.clone(),
            checksum_declared: entry.spec.source.checksum_sha256.is_some(),
            signature_declared: entry.spec.source.signature.is_some(),
        },
        featured: featured_packages.iter().any(|p| p == &entry.spec.name),
    }
}

pub fn read_registry_metadata(
    registry_name: &str,
    registry_root: &Path,
) -> Result<RegistryMetadata> {
    let path = registry_root.join("registry.yaml");
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read registry metadata {}", path.display()))?;
    let mut metadata: RegistryMetadata = serde_yaml::from_str(&contents)
        .with_context(|| format!("failed to parse registry metadata {}", path.display()))?;
    if metadata.name.trim().is_empty() {
        metadata.name = registry_name.to_string();
    }
    Ok(metadata)
}

pub fn default_registry_metadata(registry_name: &str, registry_root: &Path) -> RegistryMetadata {
    RegistryMetadata {
        schema_version: "0.3".to_string(),
        name: registry_name.to_string(),
        title: format!("{} local source", registry_name),
        revision: None,
        generated_at: None,
        featured_packages: None,
        provenance: RegistryMetadataProvenance {
            maintainer: "unknown".to_string(),
            trust_policy: "metadata-only".to_string(),
            source: registry_root.to_string_lossy().to_string(),
        },
    }
}
