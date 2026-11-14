//! WinGet is a separate external installer, never part of native Trellis receipts.
//! Downloaded browser plans are untrusted selections, not executable commands.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::batch::SelectionPlan;

const MAX_CATALOG_BYTES: u64 = 2 * 1024 * 1024;
const MAX_WINGET_OUTPUT: usize = 256 * 1024;
const MAX_SELECTIONS: usize = 100;

trait WingetBackend {
    fn version(&self) -> Result<String>;
    fn show(&self, id: &str) -> Result<String>;
    fn install(&self, id: &str) -> Result<i32>;
}

struct SystemWinget;

impl SystemWinget {
    fn output(args: &[&str]) -> Result<String> {
        let output = Command::new("winget.exe")
            .args(args)
            .output()
            .context("could not start WinGet; confirm App Installer / winget is available")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "WinGet preflight failed (exit {:?}): {}",
                output.status.code(),
                stderr.trim()
            );
        }
        if output.stdout.len() > MAX_WINGET_OUTPUT || output.stderr.len() > MAX_WINGET_OUTPUT {
            bail!("WinGet preflight response exceeds safety limit");
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

impl WingetBackend for SystemWinget {
    fn version(&self) -> Result<String> {
        Self::output(&["--version"])
    }

    fn show(&self, id: &str) -> Result<String> {
        Self::output(&[
            "show",
            "--id",
            id,
            "--exact",
            "--source",
            "winget",
            "--disable-interactivity",
        ])
    }

    fn install(&self, id: &str) -> Result<i32> {
        // No shell, no installer override, no remote script, no automatic agreement acceptance.
        // --no-upgrade prevents silently upgrading an existing package.
        let status = Command::new("winget.exe")
            .args([
                "install",
                "--id",
                id,
                "--exact",
                "--source",
                "winget",
                "--no-upgrade",
                "--disable-interactivity",
            ])
            .status()
            .with_context(|| format!("failed to start WinGet install for {}", id))?;
        Ok(status.code().unwrap_or(-1))
    }
}

#[derive(Debug, Serialize)]
struct WingetReceipt {
    schema_version: &'static str,
    backend: &'static str,
    source: &'static str,
    package_id: String,
    executed_at_utc: String,
    status: &'static str,
    exit_code: i32,
    approval_hash: String,
    note: &'static str,
}

pub fn run(
    plan: &Path,
    catalog: &Path,
    home: &Path,
    apply: bool,
    confirm: Option<&str>,
) -> Result<()> {
    if !cfg!(target_os = "windows") {
        bail!("WinGet batch execution and preflight are supported on Windows only");
    }
    run_with(&SystemWinget, plan, catalog, home, apply, confirm).map(|_| ())
}

fn is_valid_id(id: &str) -> bool {
    id.len() >= 3
        && id.len() <= 100
        && id.is_ascii()
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}

fn catalog_allowlist(path: &Path) -> Result<Vec<u8>> {
    let metadata =
        fs::metadata(path).with_context(|| format!("cannot inspect catalog {}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_CATALOG_BYTES {
        bail!("catalog must be a regular JSON file no larger than 2 MiB");
    }
    fs::read(path).with_context(|| format!("cannot read catalog {}", path.display()))
}

fn selected_ids(plan: &SelectionPlan, catalog_bytes: &[u8]) -> Result<Vec<String>> {
    let catalog: serde_json::Value =
        serde_json::from_slice(catalog_bytes).context("invalid local App Center catalog JSON")?;
    if catalog["schema_version"] != "trellis.app-center.v0.1" {
        bail!("unsupported App Center catalog schema");
    }
    let apps = catalog["apps"]
        .as_array()
        .context("catalog apps must be an array")?;
    let mut allowlist = BTreeMap::new();
    let mut catalog_duplicates = BTreeSet::new();

    for app in apps {
        if app["kind"] != "winget" {
            continue;
        }
        if app["status"] != "external_preflight" {
            continue;
        }
        if !app["platforms"]
            .as_array()
            .is_some_and(|a| a.iter().any(|p| p == "windows"))
        {
            continue;
        }
        let Some(id) = app["package_id"].as_str() else {
            continue;
        };
        let Some(entry_id) = app["id"].as_str() else {
            continue;
        };
        if !is_valid_id(id) || entry_id != format!("winget:{}", id) {
            bail!("catalog contains malformed WinGet entry");
        }
        if !catalog_duplicates.insert(entry_id.to_ascii_lowercase()) {
            bail!("catalog contains duplicate WinGet entry");
        }
        allowlist.insert(entry_id.to_string(), id.to_string());
    }

    if plan.items.len() > MAX_SELECTIONS {
        bail!("WinGet batch is limited to 100 selections");
    }
    let mut ids = BTreeSet::new();
    for item in &plan.items {
        if item.kind != "winget" || item.status != "external_preflight" {
            bail!(
                "HELD {}: WinGet adapter accepts only a pure Windows WinGet plan",
                item.id
            );
        }
        let id = allowlist.get(&item.id).with_context(|| {
            format!(
                "HELD {}: ID not in the reviewed local WinGet catalog",
                item.id
            )
        })?;
        ids.insert(id.clone());
    }
    if ids.is_empty() {
        bail!("no approved catalog candidates selected");
    }
    Ok(ids.into_iter().collect())
}

fn show_contains_exact_id(output: &str, id: &str) -> bool {
    output.lines().any(|line| {
        let l = line.trim();
        l.contains(&format!("[{}]", id))
            || (l.contains(':') && l.rsplit(':').next().is_some_and(|v| v.trim() == id))
    })
}

fn fingerprint(plan: &[u8], catalog: &[u8], version: &str, shows: &[(String, String)]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"trellis.winget-batch.v0.1\0");
    for chunk in [plan, catalog, version.as_bytes()] {
        hash.update((chunk.len() as u64).to_be_bytes());
        hash.update(chunk);
    }
    for (id, evidence) in shows {
        for chunk in [id.as_bytes(), evidence.as_bytes()] {
            hash.update((chunk.len() as u64).to_be_bytes());
            hash.update(chunk);
        }
    }
    format!("{:x}", hash.finalize())
}

fn write_receipt(home: &Path, hash: &str, index: usize, id: &str, code: i32) -> Result<PathBuf> {
    let root = home.join("external").join("winget").join(hash);
    fs::create_dir_all(&root).context("could not create local WinGet receipt directory")?;
    let path = root.join(format!("{:03}-{}.json", index + 1, id));
    let receipt = WingetReceipt {
        schema_version: "trellis.winget-receipt.v0.1",
        backend: "winget",
        source: "winget",
        package_id: id.to_string(),
        executed_at_utc: Utc::now().to_rfc3339(),
        status: if code == 0 { "winget_exit_zero_unverified" } else { "winget_exit_nonzero" },
        exit_code: code,
        approval_hash: hash.to_string(),
        note: "Records WinGet process exit only. Not proof of installed software, rollback, license, or safety.",
    };
    fs::write(&path, serde_json::to_vec_pretty(&receipt)?)
        .with_context(|| format!("failed to write local receipt {}", path.display()))?;
    Ok(path)
}

fn run_with(
    winget: &impl WingetBackend,
    plan_file: &Path,
    catalog_file: &Path,
    home: &Path,
    apply: bool,
    confirm: Option<&str>,
) -> Result<String> {
    if !apply && confirm.is_some() {
        bail!("--confirm is only valid with --apply");
    }
    let (plan, plan_bytes) = SelectionPlan::from_file(plan_file)?;
    let catalog_bytes = catalog_allowlist(catalog_file)?;
    let ids = selected_ids(&plan, &catalog_bytes)?;

    println!("Φ Trellis WinGet Batch · {} candidates", ids.len());
    println!("Source fixed to winget; version/publisher are NOT pinned by this adapter.");
    println!(
        "No automatic license acceptance; blocked source terms require separate operator action."
    );
    let version = winget.version()?;
    println!("WinGet version: {}", version.trim());

    let mut observations = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        println!(
            "Preflight [{}/{}]: winget show --id {} --exact --source winget",
            i + 1,
            ids.len(),
            id
        );
        let output = winget
            .show(id)
            .with_context(|| format!("preflight for {} failed", id))?;
        if !show_contains_exact_id(&output, id) {
            bail!(
                "HELD {}: WinGet show did not display the exact expected package ID",
                id
            );
        }
        // Display the actual source metadata before the operator approves the batch.
        // Remove terminal control characters from lower-trust package descriptions.
        let readable: String = output
            .chars()
            .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
            .collect();
        println!("WinGet metadata for {}:\n{}", id, readable.trim_end());
        observations.push((id.clone(), output));
    }
    let approval = fingerprint(&plan_bytes, &catalog_bytes, &version, &observations);
    println!("Approval hash: {}", approval);

    if !apply {
        println!("DRY RUN: no installer was started and no Trellis receipt was written.");
        println!("Preflight may contact the configured WinGet source and modify its caches.");
        println!(
            "To install: trellis winget-batch --plan FILE --apply --confirm FULL_APPROVAL_HASH"
        );
        return Ok(approval);
    }
    if confirm != Some(approval.as_str()) {
        bail!("WinGet batch refused: exact approval hash does not match current preflight");
    }

    println!(
        "Approved {} WinGet candidates. Installing sequentially.",
        ids.len()
    );
    for (i, id) in ids.iter().enumerate() {
        println!("Installing [{}/{}]: {}", i + 1, ids.len(), id);
        let code = winget.install(id)?;
        let receipt_path = write_receipt(home, &approval, i, id, code)?;
        println!(
            "WinGet exit code {} for {}; local record: {}",
            code,
            id,
            receipt_path.display()
        );
        if code != 0 {
            bail!(
                "WinGet reported failure for {}; stopping remaining installations",
                id
            );
        }
    }
    println!("WinGet batch finished. Review applications in Windows and verify installed state separately.");
    Ok(approval)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use tempfile::tempdir;

    struct FakeWinget {
        installs: Cell<usize>,
        shows: Cell<usize>,
        failure_code: i32,
    }
    impl WingetBackend for FakeWinget {
        fn version(&self) -> Result<String> {
            Ok("v1.12.0\n".to_string())
        }
        fn show(&self, id: &str) -> Result<String> {
            self.shows.set(self.shows.get() + 1);
            Ok(format!(
                "Found Example [{}]\nVersion: 1.2.3\nPublisher: Test\n",
                id
            ))
        }
        fn install(&self, _id: &str) -> Result<i32> {
            self.installs.set(self.installs.get() + 1);
            Ok(self.failure_code)
        }
    }

    fn fixture(ids: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempdir().unwrap();
        let plan = dir.path().join("plan.json");
        let catalog = dir.path().join("catalog.json");
        let items: Vec<serde_json::Value> = ids.iter().map(|(id, kind)| serde_json::json!({
            "id": format!("{}:{}", kind, id),
            "kind": kind,
            "status": if *kind == "winget" { "external_preflight" } else { "review_required" }
        })).collect();
        fs::write(
            &plan,
            serde_json::to_vec(&serde_json::json!({
                "schema_version": "trellis.selection-plan.v0.1",
                "approval": "not_granted", "execution": "none", "items": items
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            &catalog,
            serde_json::to_vec(&serde_json::json!({
                "schema_version": "trellis.app-center.v0.1",
                "apps": [
                    {"id":"winget:Git.Git", "kind":"winget", "status":"external_preflight",
                      "package_id":"Git.Git","platforms":["windows"]},
                    {"id":"winget:VideoLAN.VLC", "kind":"winget", "status":"external_preflight",
                      "package_id":"VideoLAN.VLC","platforms":["windows"]}
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        (dir, plan, catalog)
    }

    fn backend() -> FakeWinget {
        FakeWinget {
            installs: Cell::new(0),
            shows: Cell::new(0),
            failure_code: 0,
        }
    }

    #[test]
    fn preview_has_no_install_effects() {
        let (d, p, c) = fixture(&[("Git.Git", "winget")]);
        let b = backend();
        let hash = run_with(&b, &p, &c, d.path(), false, None).unwrap();
        assert_eq!(hash.len(), 64);
        assert_eq!(b.installs.get(), 0);
        assert!(!d.path().join("external").exists());
    }

    #[test]
    fn mismatch_prevents_install() {
        let (d, p, c) = fixture(&[("Git.Git", "winget")]);
        let b = backend();
        assert!(run_with(&b, &p, &c, d.path(), true, Some("incorrect")).is_err());
        assert_eq!(b.installs.get(), 0);
    }

    #[test]
    fn mixed_providers_block_before_process_execution() {
        let (d, p, c) = fixture(&[("Git.Git", "winget"), ("BTM", "phi_repo")]);
        let b = backend();
        assert!(run_with(&b, &p, &c, d.path(), true, Some("bad")).is_err());
        assert_eq!(b.shows.get(), 0);
        assert_eq!(b.installs.get(), 0);
    }

    #[test]
    fn unlisted_package_id_blocks_execution() {
        let (d, p, c) = fixture(&[("Unknown.App", "winget")]);
        let b = backend();
        assert!(run_with(&b, &p, &c, d.path(), true, Some("bad")).is_err());
        assert_eq!(b.shows.get(), 0);
    }

    #[test]
    fn altered_catalog_invalidates_approval() {
        let (d, p, c) = fixture(&[("Git.Git", "winget")]);
        let b = backend();
        let hash = run_with(&b, &p, &c, d.path(), false, None).unwrap();
        let original = fs::read_to_string(&c).unwrap();
        fs::write(&c, format!("{} ", original)).unwrap();
        assert!(run_with(&b, &p, &c, d.path(), true, Some(&hash)).is_err());
        assert_eq!(b.installs.get(), 0);
    }

    #[test]
    fn explicit_confirm_records_each_success() {
        let (d, p, c) = fixture(&[("Git.Git", "winget"), ("VideoLAN.VLC", "winget")]);
        let b = backend();
        let hash = run_with(&b, &p, &c, d.path(), false, None).unwrap();
        run_with(&b, &p, &c, d.path(), true, Some(&hash)).unwrap();
        assert_eq!(b.installs.get(), 2);
        assert!(d
            .path()
            .join("external/winget")
            .join(&hash)
            .join("001-Git.Git.json")
            .exists());
        assert!(d
            .path()
            .join("external/winget")
            .join(&hash)
            .join("002-VideoLAN.VLC.json")
            .exists());
    }

    #[test]
    fn nonzero_exit_records_failure_and_stops() {
        let (d, p, c) = fixture(&[("Git.Git", "winget"), ("VideoLAN.VLC", "winget")]);
        let mut b = backend();
        let hash = run_with(&b, &p, &c, d.path(), false, None).unwrap();
        b.failure_code = 123;
        assert!(run_with(&b, &p, &c, d.path(), true, Some(&hash)).is_err());
        assert_eq!(b.installs.get(), 1);
        assert!(d
            .path()
            .join("external/winget")
            .join(&hash)
            .join("001-Git.Git.json")
            .exists());
        assert!(!d
            .path()
            .join("external/winget")
            .join(&hash)
            .join("002-VideoLAN.VLC.json")
            .exists());
    }
}
