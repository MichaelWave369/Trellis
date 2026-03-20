use assert_cmd::Command;
use predicates::str::contains;
use std::process::Command as ProcessCommand;
use tempfile::tempdir;

#[test]
fn search_discovers_all_featured_native_packages() {
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
        .args(["search", "cli"])
        .assert()
        .success()
        .stdout(contains("overstrings-cli"))
        .stdout(contains("featured"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["search", "pulse"])
        .assert()
        .success()
        .stdout(contains("tiekat-pulse"));
}

#[test]
fn info_and_inspect_are_distinct_for_native_packages() {
    let home = tempdir().unwrap();
    let registry_root = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["init"])
        .assert()
        .success();

    for pkg in ["overstrings-cli", "vineyard-core", "tiekat-pulse"] {
        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry_root)
            .args(["info", pkg])
            .assert()
            .success()
            .stdout(contains("== Package Info =="))
            .stdout(contains("Signature"));

        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry_root)
            .args(["inspect", pkg])
            .assert()
            .success()
            .stdout(contains("== Inspect Package Spec =="))
            .stdout(contains("Dependencies"));
    }
}

#[test]
fn install_list_remove_flow_works_for_all_native_packages_and_binaries_run() {
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

    for pkg in ["vineyard-core", "overstrings-cli", "tiekat-pulse"] {
        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry_root)
            .args(["install", pkg])
            .assert()
            .success()
            .stdout(contains("Resolution summary"));
    }

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["list"])
        .assert()
        .success()
        .stdout(contains("overstrings-cli"))
        .stdout(contains("vineyard-core"))
        .stdout(contains("tiekat-pulse"));

    let over_out = ProcessCommand::new(home.path().join("bin/overstrings"))
        .args(["normalize", "Hello Trellis"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&over_out.stdout).contains("hello-trellis"));

    let vineyard_out = ProcessCommand::new(home.path().join("bin/vineyard-core"))
        .arg("status")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&vineyard_out.stdout).contains("vineyard-core 0.6.0"));

    let pulse_out = ProcessCommand::new(home.path().join("bin/tiekat-pulse"))
        .arg("version")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&pulse_out.stdout).contains("tiekat-pulse 0.6.0"));

    for pkg in ["overstrings-cli", "tiekat-pulse", "vineyard-core"] {
        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry_root)
            .args(["remove", pkg])
            .assert()
            .success();
    }
}
