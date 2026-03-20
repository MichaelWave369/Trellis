use anyhow::{bail, Result};
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
    if install_root.exists() {
        fs::remove_dir_all(&install_root)?;
    }

    for cmd in receipt.exposed_binaries.keys() {
        let bin = paths.bin.join(cmd);
        if bin.exists() {
            let _ = fs::remove_file(&bin);
        }
    }

    fs::remove_file(receipt_path)?;
    Ok(())
}
