use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn search_finds_vineyard_core_and_info_prints_registry_metadata() {
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

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["search", "vineyard"])
        .assert()
        .success()
        .stdout(contains("vineyard-core 0.2.0 [binary]"))
        .stdout(contains("registry: vineyard-core"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["info", "vineyard-core"])
        .assert()
        .success()
        .stdout(contains("Resolved registry: vineyard-core"))
        .stdout(contains("Integrity: checksum=absent signature=dev-local"));
}
