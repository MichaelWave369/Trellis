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

    for dir in [
        "cache",
        "cellar",
        "receipts",
        "registry",
        "registry/cache",
        "bin",
    ] {
        assert!(home.path().join(dir).exists(), "missing {}", dir);
    }
}

#[test]
fn init_creates_default_registry_sources_manifest() {
    let home = tempdir().unwrap();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("init")
        .assert()
        .success();

    assert!(home.path().join("registry/sources.json").exists());
}
