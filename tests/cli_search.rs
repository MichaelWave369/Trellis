use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn search_info_output_are_polished_and_trust_visible() {
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
        .stdout(contains("== Search Packages =="))
        .stdout(contains("Name"))
        .stdout(contains("vineyard-core"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["info", "vineyard-core"])
        .assert()
        .success()
        .stdout(contains("== Package Info =="))
        .stdout(contains("Signature     : Present"));
}

#[test]
fn search_no_results_shows_guidance() {
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
        .args(["search", "definitely-no-hit"])
        .assert()
        .success()
        .stdout(contains("No packages matched your query"))
        .stdout(contains("Try a broader query"));
}
