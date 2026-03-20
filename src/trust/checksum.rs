use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
