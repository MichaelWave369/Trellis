use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn install_resolves_dependencies_and_writes_lock_state() {
    let home = tempdir().unwrap();
    let registry_root = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));

    for args in [["init"], ["update"]] {
        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry_root)
            .args(args)
            .assert()
            .success();
    }

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .arg("--profile")
        .arg("default")
        .args(["install", "overstrings-cli"])
        .assert()
        .success()
        .stdout(contains("Resolution order"))
        .stdout(contains("Lock state written"));

    assert!(home.path().join("receipts/overstrings-cli.json").exists());
    assert!(home.path().join("receipts/vineyard-core.json").exists());

    let lock = fs::read_to_string(home.path().join("locks/default.lock.json")).unwrap();
    assert!(lock.contains("overstrings-cli"));
    assert!(lock.contains("vineyard-core"));
}

#[test]
fn verify_detects_drift_and_repair_restores_binaries() {
    let home = tempdir().unwrap();
    let registry_root = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));

    for args in [&["seed"][..], &["install", "vineyard-core"][..]] {
        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry_root)
            .args(args)
            .assert()
            .success();
    }

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["verify"])
        .assert()
        .success();

    fs::remove_file(home.path().join("bin/vineyard-core")).unwrap();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["verify"])
        .assert()
        .failure()
        .stdout(contains("missing exposed binary"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["repair"])
        .assert()
        .success()
        .stdout(contains("Repair completed"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["verify"])
        .assert()
        .success();
}
