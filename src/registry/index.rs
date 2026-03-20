use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::spec::{load_spec, package::PackageSpec};

#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub spec_path: PathBuf,
    pub spec: PackageSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub spec_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIndex {
    pub packages: Vec<IndexedPackage>,
}

pub fn scan_registry(registry_root: &Path) -> Result<Vec<RegistryEntry>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(registry_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
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
            let spec = load_spec(path)?;
            out.push(RegistryEntry {
                spec_path: path.to_path_buf(),
                spec,
            });
        }
    }

    out.sort_by(|a, b| a.spec.name.cmp(&b.spec.name));
    Ok(out)
}

pub fn write_index(entries: &[RegistryEntry], index_path: &Path) -> Result<()> {
    let index = RegistryIndex {
        packages: entries
            .iter()
            .map(|e| IndexedPackage {
                name: e.spec.name.clone(),
                version: e.spec.version.clone(),
                description: e.spec.description.clone(),
                spec_path: e.spec_path.to_string_lossy().to_string(),
            })
            .collect(),
    };
    let serialized = serde_json::to_string_pretty(&index)?;
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
