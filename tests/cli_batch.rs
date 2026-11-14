use assert_cmd::Command;
use predicates::str::contains;
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::{tempdir, TempDir};

fn setup() -> (TempDir, String) {
    let home = tempdir().unwrap();
    let registry = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));
    for step in ["init", "update"] {
        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry)
            .arg(step)
            .assert()
            .success();
    }
    (home, registry)
}

fn write_plan(path: &Path, ids: &[(&str, &str, &str)]) {
    let items: Vec<_> = ids
        .iter()
        .map(|(id, kind, status)| {
            json!({
                "id": id,
                "name": id,
                "kind": kind,
                "status": status,
                "suggested_commands": ["echo NEVER_RUN_THIS_UNTRUSTED_FIELD"]
            })
        })
        .collect();
    let plan = json!({
        "schema_version": "trellis.selection-plan.v0.1",
        "created_at": "2026-11-13T20:00:00.000Z",
        "approval": "not_granted",
        "execution": "none",
        "items": items
    });
    fs::write(path, serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
}

fn approval_hash(home: &Path, registry: &str, plan: &Path) -> String {
    let output = Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home)
        .arg("--registry-root")
        .arg(registry)
        .args(["batch", "--plan"])
        .arg(plan)
        .output()
        .unwrap();
    assert!(output.status.success(), "preview failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let prefix = "Approval hash: ";
    let result = stdout
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .expect("approval hash absent");
    assert_eq!(result.len(), 64);
    result.to_string()
}

#[test]
fn dry_run_does_not_create_install_receipts_or_lock() {
    let (home, registry) = setup();
    let file = home.path().join("choice.json");
    write_plan(
        &file,
        &[("trellis:overstrings-cli", "trellis", "native_catalog")],
    );
    let hash = approval_hash(home.path(), &registry, &file);
    assert_eq!(hash.len(), 64);
    assert!(!home.path().join("receipts/overstrings-cli.json").exists());
    assert!(!home.path().join("receipts/vineyard-core.json").exists());
    assert!(!home.path().join("locks/default.lock.json").exists());
}

#[test]
fn incorrect_digest_does_not_install_anything() {
    let (home, registry) = setup();
    let file = home.path().join("choice.json");
    write_plan(
        &file,
        &[("trellis:overstrings-cli", "trellis", "native_catalog")],
    );
    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry)
        .args(["batch", "--plan"])
        .arg(&file)
        .args(["--apply", "--confirm", "not-the-valid-digest"])
        .assert()
        .failure()
        .stderr(contains("batch refused"));
    assert!(!home.path().join("receipts/overstrings-cli.json").exists());
    assert!(!home.path().join("receipts/vineyard-core.json").exists());
}

#[test]
fn mixed_provider_plan_is_held_before_native_install() {
    let (home, registry) = setup();
    let file = home.path().join("choice.json");
    write_plan(
        &file,
        &[
            ("trellis:vineyard-core", "trellis", "native_catalog"),
            ("winget:Google.Chrome", "winget", "external_preflight"),
        ],
    );
    let hash = approval_hash(home.path(), &registry, &file);
    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry)
        .args(["batch", "--plan"])
        .arg(&file)
        .args(["--apply", "--confirm"])
        .arg(hash)
        .assert()
        .failure()
        .stderr(contains("batch refused"));
    assert!(!home.path().join("receipts/vineyard-core.json").exists());
}

#[test]
fn changed_plan_invalidates_approval_hash() {
    let (home, registry) = setup();
    let file = home.path().join("choice.json");
    write_plan(
        &file,
        &[("trellis:vineyard-core", "trellis", "native_catalog")],
    );
    let before = approval_hash(home.path(), &registry, &file);
    let raw = fs::read_to_string(&file).unwrap();
    fs::write(&file, format!("{}\n", raw)).unwrap();
    let after = approval_hash(home.path(), &registry, &file);
    assert_ne!(before, after);
    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry)
        .args(["batch", "--plan"])
        .arg(&file)
        .args(["--apply", "--confirm"])
        .arg(before)
        .assert()
        .failure()
        .stderr(contains("batch refused"));
}

#[test]
#[cfg(target_os = "linux")]
fn approved_native_batch_installs_dependency_and_preserves_profile_lock() {
    let (home, registry) = setup();
    let file = home.path().join("choice.json");
    write_plan(
        &file,
        &[("trellis:overstrings-cli", "trellis", "native_catalog")],
    );
    let hash = approval_hash(home.path(), &registry, &file);
    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry)
        .args(["batch", "--plan"])
        .arg(&file)
        .args(["--apply", "--confirm"])
        .arg(&hash)
        .assert()
        .success()
        .stdout(contains("Native batch finished"));
    assert!(home.path().join("receipts/overstrings-cli.json").exists());
    assert!(home.path().join("receipts/vineyard-core.json").exists());
    let lock: serde_json::Value =
        serde_json::from_slice(&fs::read(home.path().join("locks/default.lock.json")).unwrap())
            .unwrap();
    assert_eq!(lock["packages"].as_array().unwrap().len(), 2);

    // Re-running an approved plan should be idempotent for the same versions.
    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry)
        .args(["batch", "--plan"])
        .arg(&file)
        .args(["--apply", "--confirm"])
        .arg(&hash)
        .assert()
        .success()
        .stdout(contains("Already installed"));
}
