use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn remove_cleans_installed_state() {
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
        .args(["remove", "vineyard-core"])
        .assert()
        .success();

    assert!(!home.path().join("receipts/vineyard-core.json").exists());
}
