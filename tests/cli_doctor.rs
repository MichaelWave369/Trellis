use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn doctor_reports_healthy_environment() {
    let home = tempdir().unwrap();
    let registry_root = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));

    for args in [["init"], ["update"], ["install", "vineyard-core"]] {
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
        .args(["doctor"])
        .assert()
        .success()
        .stdout(contains("Environment is healthy"));
}
