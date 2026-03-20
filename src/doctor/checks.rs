use anyhow::Result;
use chrono::{Duration, Utc};
use std::collections::HashSet;
use std::fs;

use crate::core::paths::TrellisPaths;
use crate::core::receipts::read_receipt;
use crate::registry::config::read_registry_config;
use crate::registry::index::read_index;

#[derive(Debug)]
pub struct CheckReport {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

pub fn run_checks(paths: &TrellisPaths) -> Vec<CheckReport> {
    vec![
        check_dirs(paths),
        check_registry_config(paths),
        check_registry_index(paths),
        check_registry_integrity(paths),
        check_receipts(paths),
        check_binaries(paths),
    ]
}

pub fn report_counts(reports: &[CheckReport]) -> (usize, usize) {
    let passed = reports.iter().filter(|r| r.ok).count();
    (passed, reports.len().saturating_sub(passed))
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
            ok: true,
            detail: "all required Trellis directories exist".to_string(),
        }
    } else {
        CheckReport {
            name: "directories",
            ok: false,
            detail: format!("missing: {}", missing.join(", ")),
        }
    }
}

fn check_registry_config(paths: &TrellisPaths) -> CheckReport {
    match read_registry_config(&paths.registry_sources) {
        Ok(config) => {
            let enabled = config.sources.into_iter().filter(|s| s.enabled).count();
            CheckReport {
                name: "registry config",
                ok: enabled > 0,
                detail: format!("{} enabled source(s)", enabled),
            }
        }
        Err(err) => CheckReport {
            name: "registry config",
            ok: false,
            detail: err.to_string(),
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
                ok: !stale,
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
            }
        }
        Err(err) => CheckReport {
            name: "registry index",
            ok: false,
            detail: err.to_string(),
        },
    }
}

fn check_registry_integrity(paths: &TrellisPaths) -> CheckReport {
    let index = match read_index(&paths.registry_index) {
        Ok(index) => index,
        Err(err) => {
            return CheckReport {
                name: "registry health",
                ok: false,
                detail: err.to_string(),
            }
        }
    };

    let mut duplicate_guard = HashSet::new();
    for pkg in &index.packages {
        let key = format!("{}:{}:{}", pkg.registry, pkg.name, pkg.version);
        if !duplicate_guard.insert(key.clone()) {
            return CheckReport {
                name: "registry health",
                ok: false,
                detail: format!("duplicate package entry: {}", key),
            };
        }

        if !std::path::Path::new(&pkg.spec_path).exists() {
            return CheckReport {
                name: "registry health",
                ok: false,
                detail: format!("missing spec path in index: {}", pkg.spec_path),
            };
        }
    }

    CheckReport {
        name: "registry health",
        ok: index.skipped.is_empty(),
        detail: if index.skipped.is_empty() {
            format!(
                "{} indexed package(s) with no malformed entries",
                index.packages.len()
            )
        } else {
            format!(
                "{} malformed spec(s); run 'trellis update' and inspect registry/index.json",
                index.skipped.len()
            )
        },
    }
}

fn check_receipts(paths: &TrellisPaths) -> CheckReport {
    let entries = match fs::read_dir(&paths.receipts) {
        Ok(entries) => entries,
        Err(err) => {
            return CheckReport {
                name: "receipts",
                ok: false,
                detail: err.to_string(),
            }
        }
    };

    let mut parsed = 0usize;
    for entry in entries.flatten() {
        if entry.path().extension().and_then(|v| v.to_str()) == Some("json") {
            if read_receipt(&entry.path()).is_err() {
                return CheckReport {
                    name: "receipts",
                    ok: false,
                    detail: format!("failed to parse {}", entry.path().display()),
                };
            }
            parsed += 1;
        }
    }

    CheckReport {
        name: "receipts",
        ok: true,
        detail: format!("{} receipt(s) parseable", parsed),
    }
}

fn check_binaries(paths: &TrellisPaths) -> CheckReport {
    let entries = match fs::read_dir(&paths.bin) {
        Ok(entries) => entries,
        Err(err) => {
            return CheckReport {
                name: "binaries",
                ok: false,
                detail: err.to_string(),
            }
        }
    };

    let mut count = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() && !path.is_symlink() {
            return CheckReport {
                name: "binaries",
                ok: false,
                detail: format!("non-file entry in bin: {}", path.display()),
            };
        }

        if path.is_symlink() {
            match fs::read_link(&path) {
                Ok(target) => {
                    if !target.exists() {
                        return CheckReport {
                            name: "binaries",
                            ok: false,
                            detail: format!("broken link: {}", path.display()),
                        };
                    }
                }
                Err(err) => {
                    return CheckReport {
                        name: "binaries",
                        ok: false,
                        detail: err.to_string(),
                    }
                }
            }
        }

        count += 1;
    }

    CheckReport {
        name: "binaries",
        ok: true,
        detail: format!("{} exposed binary file(s) validated", count),
    }
}

pub fn summarize(reports: &[CheckReport]) -> Result<()> {
    if reports.iter().all(|r| r.ok) {
        Ok(())
    } else {
        anyhow::bail!("doctor found issues")
    }
}
