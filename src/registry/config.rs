use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub schema_version: String,
    pub sources: Vec<RegistrySource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySource {
    pub name: String,
    pub kind: RegistryKind,
    pub enabled: bool,
    pub official: bool,
    pub location: String,
    #[serde(default)]
    pub mirrors: Vec<RegistryMirror>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMirror {
    pub name: String,
    pub location: String,
    pub priority: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegistryKind {
    LocalPath,
}

impl RegistryConfig {
    pub fn default_with_registry_root(registry_root: &Path) -> Self {
        Self {
            schema_version: "0.3".to_string(),
            sources: vec![RegistrySource {
                name: "vineyard-core".to_string(),
                kind: RegistryKind::LocalPath,
                enabled: true,
                official: true,
                location: registry_root.to_string_lossy().to_string(),
                mirrors: vec![RegistryMirror {
                    name: "local-cache-placeholder".to_string(),
                    location: "file://mirror-not-yet-implemented".to_string(),
                    priority: 100,
                    enabled: false,
                }],
            }],
        }
    }
}

pub fn ensure_registry_config(path: &Path, registry_root: &Path) -> Result<RegistryConfig> {
    if path.exists() {
        return read_registry_config(path);
    }

    let config = RegistryConfig::default_with_registry_root(registry_root);
    write_registry_config(path, &config)?;
    Ok(config)
}

pub fn read_registry_config(path: &Path) -> Result<RegistryConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read registry config {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse registry config {}", path.display()))
}

pub fn write_registry_config(path: &Path, config: &RegistryConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let contents = serde_json::to_string_pretty(config)?;
    fs::write(path, contents)
        .with_context(|| format!("failed to write registry config {}", path.display()))
}

pub fn resolve_source_path(base_dir: &Path, location: &str) -> PathBuf {
    let path = PathBuf::from(location);
    if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    }
}
