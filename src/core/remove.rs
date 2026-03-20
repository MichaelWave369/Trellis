use anyhow::{bail, Context, Result};
use std::fs;

use crate::core::paths::TrellisPaths;
use crate::core::receipts::read_receipt;

pub fn remove(paths: &TrellisPaths, pkg: &str) -> Result<()> {
    let receipt_path = paths.receipts.join(format!("{}.json", pkg));
    if !receipt_path.exists() {
        bail!("package '{}' is not installed", pkg);
    }

    let receipt = read_receipt(&receipt_path)?;
    let install_root = paths.cellar.join(&receipt.name).join(&receipt.version);
    if !install_root.starts_with(&paths.cellar) {
        bail!("refusing to remove install path outside Trellis cellar");
    }

    if install_root.exists() {
        fs::remove_dir_all(&install_root)
            .with_context(|| format!("failed to remove {}", install_root.display()))?;
    }

    for cmd in receipt.exposed_binaries.keys() {
        let bin = paths.bin.join(cmd);
        if !bin.starts_with(&paths.bin) {
            bail!("refusing to remove binary path outside Trellis bin");
        }
        if bin.exists() {
            fs::remove_file(&bin)
                .with_context(|| format!("failed to remove binary {}", bin.display()))?;
        }
    }

    fs::remove_file(receipt_path)?;
    Ok(())
}
