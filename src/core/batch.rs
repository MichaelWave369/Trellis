//! Untrusted selection plans are never executable instructions.
//! Only exact, locally indexed Trellis package specifications can be applied.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::registry::index::RegistryEntry;
use crate::spec::package::SourceType;
use crate::trust::checksum;

const MAX_PLAN_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ITEMS: usize = 500;

#[derive(Debug, Deserialize)]
pub struct SelectionPlan {
    pub schema_version: String,
    pub approval: String,
    pub execution: String,
    pub items: Vec<SelectionItem>,
}

#[derive(Debug, Deserialize)]
pub struct SelectionItem {
    pub id: String,
    pub kind: String,
    pub status: String,
}

impl SelectionPlan {
    pub fn from_file(path: &Path) -> Result<(Self, Vec<u8>)> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("cannot inspect selection plan {}", path.display()))?;
        if !metadata.is_file() || metadata.len() > MAX_PLAN_BYTES {
            bail!("selection plan must be a regular file no larger than 2 MiB");
        }
        let bytes = fs::read(path)
            .with_context(|| format!("cannot read selection plan {}", path.display()))?;
        let plan: Self = serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid selection plan JSON {}", path.display()))?;
        plan.validate()?;
        Ok((plan, bytes))
    }

    fn validate(&self) -> Result<()> {
        if self.schema_version != "trellis.selection-plan.v0.1" {
            bail!("unsupported selection plan schema");
        }
        if self.approval != "not_granted" || self.execution != "none" {
            bail!("selection plans must be unapproved and unexecuted; JSON cannot grant authority");
        }
        if self.items.is_empty() || self.items.len() > MAX_ITEMS {
            bail!("selection plan must contain 1 to 500 items");
        }
        let mut seen = BTreeSet::new();
        for item in &self.items {
            if !seen.insert(item.id.to_ascii_lowercase()) {
                bail!("duplicate selection ID '{}'", item.id);
            }
            match item.kind.as_str() {
                "trellis" => {
                    let name = item
                        .id
                        .strip_prefix("trellis:")
                        .context("native Trellis selection has incorrect ID prefix")?;
                    crate::spec::validate::validate_name(name)?;
                    if item.status != "native_catalog" {
                        bail!("native Trellis selection has incorrect catalog status");
                    }
                }
                "winget" | "flatpak" => {
                    if !item.id.starts_with(&format!("{}:", item.kind))
                        || item.status != "external_preflight"
                    {
                        bail!("external selection has mismatched ID, kind, or status");
                    }
                }
                "phi_repo" => {
                    if !item.id.starts_with("phi:")
                        || !matches!(item.status.as_str(), "review_required" | "archived_review")
                    {
                        bail!("Phi repository selection has mismatched ID, kind, or status");
                    }
                }
                _ => bail!("unknown selection kind: '{}'", item.kind),
            }
            if item.id.len() > 160
                || item.id.len() < 4
                || !item.id.bytes().all(|b| {
                    b.is_ascii_alphanumeric() || matches!(b, b':' | b'.' | b'-' | b'_' | b'+')
                })
            {
                bail!("invalid selection identifier");
            }
        }
        Ok(())
    }
}

/// Bind approval to raw plan bytes, exact local package specs, and actual payload digests.
/// Browser-provided shell commands, names and URLs are deliberately ignored.
pub fn approval_hash(raw: &[u8], entries: &[RegistryEntry]) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(b"trellis.native-batch.v0.1\0");
    add_bytes(&mut hasher, raw);

    for entry in entries {
        add_bytes(&mut hasher, entry.spec.name.as_bytes());
        let spec_bytes = fs::read(&entry.spec_path).with_context(|| {
            format!(
                "cannot read local package spec {}",
                entry.spec_path.display()
            )
        })?;
        add_bytes(&mut hasher, &spec_bytes);
        let source = entry
            .spec_path
            .parent()
            .context("local spec has no parent directory")?
            .join(&entry.spec.source.path);

        let expected = entry
            .spec
            .source
            .checksum_sha256
            .as_deref()
            .context("native batch requires a declared SHA-256 checksum")?;
        let actual = match entry.spec.source.source_type {
            SourceType::Dir => checksum::sha256_dir(&source)?,
            SourceType::File | SourceType::Archive => checksum::sha256_file(&source)?,
        };
        if actual != expected.to_ascii_lowercase() {
            bail!("payload checksum mismatch for '{}'", entry.spec.name);
        }
        add_bytes(&mut hasher, actual.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn add_bytes(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_json(value: serde_json::Value) -> Result<SelectionPlan> {
        let plan: SelectionPlan = serde_json::from_value(value)?;
        plan.validate()?;
        Ok(plan)
    }

    fn example() -> serde_json::Value {
        serde_json::json!({
            "schema_version": "trellis.selection-plan.v0.1",
            "approval": "not_granted", "execution": "none",
            "items": [{"id": "trellis:vineyard-core", "kind": "trellis", "status": "native_catalog"}]
        })
    }

    #[test]
    fn accepts_browser_plan_shape() {
        assert!(from_json(example()).is_ok());
    }

    #[test]
    fn rejects_duplicate_items() {
        let mut payload = example();
        let duplicated = payload["items"][0].clone();
        payload["items"].as_array_mut().unwrap().push(duplicated);
        assert!(from_json(payload).is_err());
    }

    #[test]
    fn rejects_embedded_approval() {
        let mut payload = example();
        payload["approval"] = serde_json::json!("granted");
        assert!(from_json(payload).is_err());
    }

    #[test]
    fn rejects_kind_spoofing() {
        let mut payload = example();
        payload["items"][0]["kind"] = serde_json::json!("winget");
        assert!(from_json(payload).is_err());
    }

    #[test]
    fn rejects_command_like_package_ids() {
        let mut payload = example();
        payload["items"][0]["id"] = serde_json::json!("trellis:../../danger");
        assert!(from_json(payload).is_err());
    }
}
