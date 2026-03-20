use anyhow::Result;
use std::fs;

use crate::core::paths::TrellisPaths;
use crate::core::receipts::read_receipt;
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
        check_registry_index(paths),
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

fn check_registry_index(paths: &TrellisPaths) -> CheckReport {
    let path = paths.registry.join("index.json");
    match read_index(&path) {
        Ok(index) => CheckReport {
            name: "registry index",
            ok: true,
            detail: format!("readable ({} packages)", index.packages.len()),
        },
        Err(err) => CheckReport {
            name: "registry index",
            ok: false,
            detail: err.to_string(),
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
