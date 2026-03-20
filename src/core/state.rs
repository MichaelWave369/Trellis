use anyhow::{Context, Result};
use std::fs;

use super::paths::TrellisPaths;
use crate::registry::config::ensure_registry_config;

pub fn init(paths: &TrellisPaths) -> Result<()> {
    for dir in paths.all_dirs() {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory {}", dir.display()))?;
    }

    ensure_registry_config(&paths.registry_sources, &paths.default_registry_root)?;
    Ok(())
}
