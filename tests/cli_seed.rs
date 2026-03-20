use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn seed_bootstraps_first_run_and_prints_guidance() {
    let home = tempdir().unwrap();
    let registry_root = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["seed"])
        .assert()
        .success()
        .stdout(contains("== Trellis Seed =="))
        .stdout(contains("Recommended first package: vineyard-core"))
        .stdout(contains("Trellis home"))
        .stdout(contains("Install now:"));

    assert!(home.path().join("registry/index.json").exists());
    assert!(home.path().join("registry/sources.json").exists());
}

#[test]
fn seed_is_idempotent_and_bootstrap_alias_works() {
    let home = tempdir().unwrap();
    let registry_root = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));

    for cmd in ["seed", "bootstrap", "seed"] {
        Command::cargo_bin("trellis")
            .unwrap()
            .arg("--home")
            .arg(home.path())
            .arg("--registry-root")
            .arg(&registry_root)
            .args([cmd])
            .assert()
            .success()
            .stdout(contains("Seed flow complete"));
    }
}
