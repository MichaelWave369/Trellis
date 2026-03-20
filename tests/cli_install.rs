use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn install_creates_receipt_binary_and_list_output() {
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
        .args(["install", "vineyard-core"])
        .assert()
        .success();

    assert!(home.path().join("receipts/vineyard-core.json").exists());
    assert!(home.path().join("bin/vineyard-core").exists());

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["list"])
        .assert()
        .success()
        .stdout(contains("vineyard-core"));
}
