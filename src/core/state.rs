use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use super::paths::TrellisPaths;
use crate::registry::config::ensure_registry_config;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfilesConfig {
    schema_version: String,
    default_profile: String,
    profiles: Vec<String>,
}

pub fn init(paths: &TrellisPaths) -> Result<()> {
    for dir in paths.all_dirs() {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory {}", dir.display()))?;
    }

    ensure_registry_config(&paths.registry_sources, &paths.default_registry_root)?;
    ensure_profiles(paths)?;
    Ok(())
}

fn ensure_profiles(paths: &TrellisPaths) -> Result<()> {
    if paths.profiles.exists() {
        return Ok(());
    }

    let cfg = ProfilesConfig {
        schema_version: "0.9".to_string(),
        default_profile: "default".to_string(),
        profiles: vec![
            "default".to_string(),
            "dev".to_string(),
            "minimal".to_string(),
            "diagnostics".to_string(),
        ],
    };

    fs::write(&paths.profiles, serde_json::to_string_pretty(&cfg)?).with_context(|| {
        format!(
            "failed to write profiles config {}",
            paths.profiles.display()
        )
    })
}
