use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn init_creates_expected_structure() {
    let home = tempdir().unwrap();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("init")
        .assert()
        .success();

    for dir in ["cache", "cellar", "receipts", "registry", "bin"] {
        assert!(home.path().join(dir).exists(), "missing {}", dir);
    }
}
