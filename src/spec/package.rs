use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSpec {
    pub name: String,
    pub version: String,
    pub description: String,
    pub homepage: String,
    pub source: Source,
    pub install: Install,
    #[serde(default)]
    pub bin: BTreeMap<String, String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub provenance: Provenance,
    pub health: Option<Health>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: String,
    pub path: String,
    pub checksum_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Install {
    pub strategy: String,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub publisher: String,
    pub license: String,
    pub registry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub notes: Option<String>,
}
