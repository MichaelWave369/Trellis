use anyhow::Result;
use std::path::Path;

use crate::core::paths::TrellisPaths;

use super::index::{scan_registry, write_index};

pub fn sync_registry(paths: &TrellisPaths, registry_root: &Path) -> Result<usize> {
    let entries = scan_registry(registry_root)?;
    let index_path = paths.registry.join("index.json");
    write_index(&entries, &index_path)?;
    Ok(entries.len())
}
