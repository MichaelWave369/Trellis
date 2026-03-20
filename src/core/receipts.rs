use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::trust::{ChecksumState, SignatureAssessment};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub schema_version: String,
    pub transaction_id: String,
    pub name: String,
    pub version: String,
    pub kind: String,
    pub installed_at: DateTime<Utc>,
    pub registry: RegistryReceipt,
    pub source: SourceReceipt,
    pub provenance: ProvenanceReceipt,
    pub dependencies_declared: Vec<String>,
    pub platform_evaluated: PlatformEvaluation,
    pub installed_files: Vec<String>,
    pub exposed_binaries: BTreeMap<String, String>,
    pub post_install_actions: Vec<String>,
    pub trust: TrustSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryReceipt {
    pub name: String,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReceipt {
    pub source_type: String,
    pub source_path: String,
    pub checksum_expected_sha256: Option<String>,
    pub checksum_actual_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceReceipt {
    pub publisher: String,
    pub license: String,
    pub declared_registry: String,
    pub signature: SignatureAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEvaluation {
    pub os: String,
    pub arch: String,
    pub matched: bool,
    pub constraints_os: Vec<String>,
    pub constraints_arch: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSummary {
    pub checksum_state: ChecksumState,
    pub signature_state: crate::trust::SignatureState,
    pub warnings: Vec<String>,
    pub summary: String,
}

pub fn write_receipt(receipt_path: &Path, receipt: &Receipt) -> Result<()> {
    let contents = serde_json::to_string_pretty(receipt)?;
    fs::write(receipt_path, contents)
        .with_context(|| format!("failed to write receipt {}", receipt_path.display()))
}

pub fn read_receipt(receipt_path: &Path) -> Result<Receipt> {
    let contents = fs::read_to_string(receipt_path)
        .with_context(|| format!("failed to read receipt {}", receipt_path.display()))?;
    let receipt = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse receipt {}", receipt_path.display()))?;
    Ok(receipt)
}
