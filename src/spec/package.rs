use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSpec {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub homepage: String,
    pub kind: PackageKind,
    pub source: Source,
    pub install: Install,
    #[serde(default)]
    pub bin: BTreeMap<String, String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub provenance: Provenance,
    pub platform: Option<PlatformConstraints>,
    pub post_install: Option<PostInstall>,
    pub health: Option<Health>,
}

fn default_schema_version() -> String {
    "0.2".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    Binary,
    Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub path: String,
    pub checksum_sha256: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    #[serde(rename = "local_file")]
    File,
    #[serde(rename = "local_dir")]
    Dir,
    #[serde(rename = "local_archive")]
    Archive,
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
pub struct PlatformConstraints {
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub arch: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInstall {
    pub policy: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub notes: Option<String>,
}
