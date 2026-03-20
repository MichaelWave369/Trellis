use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn receipt_command_renders_human_readable_install_ledger() {
    let home = tempdir().unwrap();
    let registry_root = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));

    let command_sets: &[&[&str]] = &[&["init"], &["update"], &["install", "vineyard-core"]];

    for args in command_sets {
        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry_root)
            .args(*args)
            .assert()
            .success();
    }

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["receipt", "vineyard-core"])
        .assert()
        .success()
        .stdout(contains("== Installed Receipt =="))
        .stdout(contains("Package       : vineyard-core"))
        .stdout(contains("Trust         : checksum="));
}
