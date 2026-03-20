use anyhow::{Context, Result};
use std::fs;

use super::paths::TrellisPaths;

pub fn init(paths: &TrellisPaths) -> Result<()> {
    for dir in paths.all_dirs() {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory {}", dir.display()))?;
    }
    Ok(())
}
