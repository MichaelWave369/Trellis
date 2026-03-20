use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::core::paths::TrellisPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockState {
    pub schema_version: String,
    pub profile: String,
    pub generated_at: DateTime<Utc>,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub registry: String,
}

pub fn lock_path(paths: &TrellisPaths, profile: &str) -> std::path::PathBuf {
    paths.locks.join(format!("{}.lock.json", profile))
}

pub fn write_lock(paths: &TrellisPaths, profile: &str, mut packages: Vec<LockedPackage>) -> Result<()> {
    packages.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));
    packages.dedup_by(|a, b| a.name == b.name && a.version == b.version && a.registry == b.registry);

    let lock = LockState {
        schema_version: "0.9".to_string(),
        profile: profile.to_string(),
        generated_at: Utc::now(),
        packages,
    };

    fs::write(lock_path(paths, profile), serde_json::to_string_pretty(&lock)?)
        .with_context(|| "failed to write lock state".to_string())
}

pub fn read_lock(paths: &TrellisPaths, profile: &str) -> Result<LockState> {
    let path = lock_path(paths, profile);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read lock state {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse lock state {}", path.display()))
}
