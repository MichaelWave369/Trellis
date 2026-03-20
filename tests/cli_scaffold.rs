use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn scaffold_creates_valid_package_structure_and_validate_inspect_work() {
    let workspace = tempdir().unwrap();
    let package_name = "author-tool";
    let package_dir = workspace.path().join("packages").join(package_name);
    let spec_path = package_dir.join(format!("{}.trellis.yaml", package_name));

    Command::cargo_bin("trellis")
        .unwrap()
        .current_dir(workspace.path())
        .args([
            "scaffold",
            package_name,
            "--out",
            workspace.path().join("packages").to_string_lossy().as_ref(),
        ])
        .assert()
        .success()
        .stdout(contains("Scaffold created"));

    assert!(spec_path.exists());
    assert!(package_dir.join("payload/bin").exists());

    let home = tempdir().unwrap();
    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["validate", spec_path.to_string_lossy().as_ref()])
        .assert()
        .success();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["inspect", spec_path.to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(contains("Inspect Package Spec"));
}

#[test]
fn readiness_and_install_from_scaffolded_path_work() {
    let workspace = tempdir().unwrap();
    let home = tempdir().unwrap();
    let package_name = "contrib-tool";
    let out = workspace.path().join("pkg-src");

    Command::cargo_bin("trellis")
        .unwrap()
        .args([
            "scaffold",
            package_name,
            "--out",
            out.to_string_lossy().as_ref(),
            "--kind",
            "source",
        ])
        .assert()
        .success();

    let spec_path = out
        .join(package_name)
        .join(format!("{}.trellis.yaml", package_name));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["seed"])
        .assert()
        .success();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["readiness", spec_path.to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(contains("Checklist"))
        .stdout(contains("spec validates"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .args(["install", "--from", spec_path.to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(contains("Installed contrib-tool"));
}
