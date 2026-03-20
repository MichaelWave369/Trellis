use anyhow::{bail, Result};

use super::package::PackageSpec;

pub fn validate(spec: &PackageSpec) -> Result<()> {
    if spec.name.trim().is_empty() {
        bail!("package name cannot be empty");
    }
    if spec.version.trim().is_empty() {
        bail!("package version cannot be empty");
    }
    if spec.source.source_type != "file" {
        bail!("source.type must be 'file' in v0.1");
    }
    if spec.install.strategy != "copy" {
        bail!("install.strategy must be 'copy' in v0.1");
    }
    if spec.install.entries.is_empty() {
        bail!("install.entries must contain at least one entry");
    }
    Ok(())
}
