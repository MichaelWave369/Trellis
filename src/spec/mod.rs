pub mod package;
pub mod validate;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::spec::package::PackageSpec;

pub fn load_spec(path: &Path) -> Result<PackageSpec> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read spec file {}", path.display()))?;
    let spec: PackageSpec = serde_yaml::from_str(&content)
        .with_context(|| format!("invalid spec {}", path.display()))?;
    validate::validate(&spec)?;
    Ok(spec)
}
