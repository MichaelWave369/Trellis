use anyhow::Result;
use chrono::{Duration, Utc};
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::core::paths::TrellisPaths;
use crate::core::receipts::read_receipt;
use crate::registry::config::read_registry_config;
use crate::registry::index::read_index;
use crate::trust::{ChecksumState, SignatureState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug)]
pub struct CheckReport {
    pub name: &'static str,
    pub status: CheckStatus,
    pub detail: String,
    pub remediation: Option<String>,
}

pub fn run_checks(paths: &TrellisPaths) -> Vec<CheckReport> {
    vec![
        check_dirs(paths),
        check_registry_config(paths),
        check_registry_index(paths),
        check_registry_integrity(paths),
        check_receipts(paths),
        check_receipt_trust(paths),
        check_binaries(paths),
    ]
}

pub fn report_counts(reports: &[CheckReport]) -> (usize, usize, usize) {
    let passed = reports
        .iter()
        .filter(|r| r.status == CheckStatus::Pass)
        .count();
    let warnings = reports
        .iter()
        .filter(|r| r.status == CheckStatus::Warn)
        .count();
    let failed = reports
        .iter()
        .filter(|r| r.status == CheckStatus::Fail)
        .count();
    (passed, warnings, failed)
}

fn check_dirs(paths: &TrellisPaths) -> CheckReport {
    let missing = paths
        .all_dirs()
        .into_iter()
        .filter(|p| !p.exists())
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>();

    if missing.is_empty() {
        CheckReport {
            name: "directories",
            status: CheckStatus::Pass,
            detail: "all required Trellis directories exist".to_string(),
            remediation: None,
        }
    } else {
        CheckReport {
            name: "directories",
            status: CheckStatus::Fail,
            detail: format!("missing: {}", missing.join(", ")),
            remediation: Some("re-run `trellis init` to recreate required directories".to_string()),
        }
    }
}

fn check_registry_config(paths: &TrellisPaths) -> CheckReport {
    match read_registry_config(&paths.registry_sources) {
        Ok(config) => {
            let enabled = config.sources.into_iter().filter(|s| s.enabled).count();
            let status = if enabled > 0 {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail
            };
            CheckReport {
                name: "registry config",
                status,
                detail: format!("{} enabled source(s)", enabled),
                remediation: (enabled == 0).then_some(
                    "enable at least one source in registry/sources.json and run `trellis update`"
                        .to_string(),
                ),
            }
        }
        Err(err) => CheckReport {
            name: "registry config",
            status: CheckStatus::Fail,
            detail: err.to_string(),
            remediation: Some("repair registry/sources.json or run `trellis init`".to_string()),
        },
    }
}

fn check_registry_index(paths: &TrellisPaths) -> CheckReport {
    match read_index(&paths.registry_index) {
        Ok(index) => {
            let age = Utc::now() - index.generated_at;
            let stale = age > Duration::days(30);
            CheckReport {
                name: "registry index",
                status: if stale {
                    CheckStatus::Warn
                } else {
                    CheckStatus::Pass
                },
                detail: if stale {
                    format!(
                        "stale index ({} packages, generated {})",
                        index.packages.len(),
                        index.generated_at
                    )
                } else {
                    format!(
                        "readable ({} packages, generated {})",
                        index.packages.len(),
                        index.generated_at
                    )
                },
                remediation: stale
                    .then_some("run `trellis update` to refresh registry metadata".to_string()),
            }
        }
        Err(err) => CheckReport {
            name: "registry index",
            status: CheckStatus::Fail,
            detail: err.to_string(),
            remediation: Some("run `trellis update` to regenerate registry/index.json".to_string()),
        },
    }
}

fn check_registry_integrity(paths: &TrellisPaths) -> CheckReport {
    let index = match read_index(&paths.registry_index) {
        Ok(index) => index,
        Err(err) => {
            return CheckReport {
                name: "registry health",
                status: CheckStatus::Fail,
                detail: err.to_string(),
                remediation: Some("run `trellis update`".to_string()),
            }
        }
    };

    let mut duplicate_guard = HashSet::new();
    for pkg in &index.packages {
        let key = format!("{}:{}:{}", pkg.registry, pkg.name, pkg.version);
        if !duplicate_guard.insert(key.clone()) {
            return CheckReport {
                name: "registry health",
                status: CheckStatus::Fail,
                detail: format!("duplicate package entry: {}", key),
                remediation: Some(
                    "clean duplicate package specs and rerun `trellis update`".to_string(),
                ),
            };
        }

        if !std::path::Path::new(&pkg.spec_path).exists() {
            return CheckReport {
                name: "registry health",
                status: CheckStatus::Fail,
                detail: format!("missing spec path in index: {}", pkg.spec_path),
                remediation: Some(
                    "run `trellis update` after repairing source registry paths".to_string(),
                ),
            };
        }
    }

    CheckReport {
        name: "registry health",
        status: if index.skipped.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        detail: if index.skipped.is_empty() {
            format!(
                "{} indexed package(s) with no malformed entries",
                index.packages.len()
            )
        } else {
            format!(
                "{} malformed spec(s) skipped during indexing",
                index.skipped.len()
            )
        },
        remediation: (!index.skipped.is_empty()).then_some(
            "inspect registry/index.json skipped entries and fix malformed package specs"
                .to_string(),
        ),
    }
}

fn check_receipts(paths: &TrellisPaths) -> CheckReport {
    let entries = match fs::read_dir(&paths.receipts) {
        Ok(entries) => entries,
        Err(err) => {
            return CheckReport {
                name: "receipts",
                status: CheckStatus::Fail,
                detail: err.to_string(),
                remediation: Some("re-run `trellis init` or repair receipts directory".to_string()),
            }
        }
    };

    let mut parsed = 0usize;
    let mut binary_claims = HashMap::<String, String>::new();
    for entry in entries.flatten() {
        if entry.path().extension().and_then(|v| v.to_str()) == Some("json") {
            let receipt = match read_receipt(&entry.path()) {
                Ok(receipt) => receipt,
                Err(_) => {
                    return CheckReport {
                        name: "receipts",
                        status: CheckStatus::Fail,
                        detail: format!("failed to parse {}", entry.path().display()),
                        remediation: Some("remove or repair malformed receipt file".to_string()),
                    }
                }
            };

            for bin in receipt.exposed_binaries.keys() {
                if let Some(owner) = binary_claims.insert(bin.clone(), receipt.name.clone()) {
                    return CheckReport {
                        name: "receipts",
                        status: CheckStatus::Fail,
                        detail: format!(
                            "binary '{}' is claimed by both '{}' and '{}'",
                            bin, owner, receipt.name
                        ),
                        remediation: Some("remove conflicting packages and reinstall".to_string()),
                    };
                }
            }
            parsed += 1;
        }
    }

    CheckReport {
        name: "receipts",
        status: CheckStatus::Pass,
        detail: format!("{} receipt(s) parseable and non-conflicting", parsed),
        remediation: None,
    }
}

fn check_receipt_trust(paths: &TrellisPaths) -> CheckReport {
    let entries = match fs::read_dir(&paths.receipts) {
        Ok(entries) => entries,
        Err(err) => {
            return CheckReport {
                name: "trust state",
                status: CheckStatus::Fail,
                detail: err.to_string(),
                remediation: Some("re-run `trellis init`".to_string()),
            }
        }
    };

    let mut warn_count = 0usize;
    let mut total = 0usize;
    for entry in entries.flatten() {
        if entry.path().extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let receipt = match read_receipt(&entry.path()) {
            Ok(receipt) => receipt,
            Err(_) => continue,
        };
        total += 1;

        if receipt.trust.checksum_state != ChecksumState::Verified {
            warn_count += 1;
        }
        if receipt.provenance.signature.state == SignatureState::Malformed {
            return CheckReport {
                name: "trust state",
                status: CheckStatus::Fail,
                detail: format!(
                    "receipt '{}' contains malformed signature metadata",
                    receipt.name
                ),
                remediation: Some("repair package signature metadata and reinstall".to_string()),
            };
        }
    }

    if warn_count == 0 {
        CheckReport {
            name: "trust state",
            status: CheckStatus::Pass,
            detail: format!(
                "{} installed package(s) with verified checksum state",
                total
            ),
            remediation: None,
        }
    } else {
        CheckReport {
            name: "trust state",
            status: CheckStatus::Warn,
            detail: format!(
                "{} of {} receipt(s) have non-verified checksum state",
                warn_count, total
            ),
            remediation: Some(
                "reinstall packages with checksum_sha256 metadata when available".to_string(),
            ),
        }
    }
}

fn check_binaries(paths: &TrellisPaths) -> CheckReport {
    let entries = match fs::read_dir(&paths.bin) {
        Ok(entries) => entries,
        Err(err) => {
            return CheckReport {
                name: "binaries",
                status: CheckStatus::Fail,
                detail: err.to_string(),
                remediation: Some("repair bin directory permissions and structure".to_string()),
            }
        }
    };

    let mut count = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() && !path.is_symlink() {
            return CheckReport {
                name: "binaries",
                status: CheckStatus::Fail,
                detail: format!("non-file entry in bin: {}", path.display()),
                remediation: Some("remove non-file entry from Trellis bin".to_string()),
            };
        }

        if path.is_symlink() {
            match fs::read_link(&path) {
                Ok(target) => {
                    if !target.exists() {
                        return CheckReport {
                            name: "binaries",
                            status: CheckStatus::Fail,
                            detail: format!("broken link: {}", path.display()),
                            remediation: Some(
                                "reinstall package that provides this binary".to_string(),
                            ),
                        };
                    }
                }
                Err(err) => {
                    return CheckReport {
                        name: "binaries",
                        status: CheckStatus::Fail,
                        detail: err.to_string(),
                        remediation: Some("repair symlink state in Trellis bin".to_string()),
                    }
                }
            }
        }

        count += 1;
    }

    CheckReport {
        name: "binaries",
        status: CheckStatus::Pass,
        detail: format!("{} exposed binary file(s) validated", count),
        remediation: None,
    }
}

pub fn summarize(reports: &[CheckReport]) -> Result<()> {
    if reports.iter().any(|r| r.status == CheckStatus::Fail) {
        anyhow::bail!("doctor found blocking issues")
    }
    Ok(())
}
