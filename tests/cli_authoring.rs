use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn validate_and_inspect_registry_package() {
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
        .args(["validate", "overstrings-cli"])
        .assert()
        .success()
        .stdout(contains("Valid: overstrings-cli"));

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["inspect", "overstrings-cli"])
        .assert()
        .success()
        .stdout(contains("Dependencies"));
}

#[test]
fn validate_rejects_invalid_name() {
    let home = tempdir().unwrap();
    let spec_dir = tempdir().unwrap();
    let spec_path = spec_dir.path().join("invalid.trellis.yaml");
    fs::write(
        &spec_path,
        r#"
schema_version: "0.2"
name: BadName
version: 1.0.0
description: bad
homepage: https://example.org/bad
kind: binary
source:
  type: local_dir
  path: payload
install:
  strategy: copy
  entries: [bin]
bin:
  bad: bin/bad
dependencies: []
provenance:
  publisher: bad
  license: MIT
  registry: vineyard-core
platform:
  os: [linux]
  arch: [x86_64]
"#,
    )
    .unwrap();

    let payload_bin = spec_dir.path().join("payload/bin");
    fs::create_dir_all(&payload_bin).unwrap();
    fs::write(payload_bin.join("bad"), "#!/usr/bin/env sh\necho bad\n").unwrap();

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
        .arg("validate")
        .arg(spec_path)
        .assert()
        .failure()
        .stderr(contains("invalid package name"));
}

#[test]
fn install_from_path_works_for_author_flow() {
    let home = tempdir().unwrap();
    let pkg_root = tempdir().unwrap();

    fs::create_dir_all(pkg_root.path().join("payload/bin")).unwrap();
    fs::write(
        pkg_root.path().join("payload/bin/local-author"),
        "#!/usr/bin/env sh\necho local-author 0.1.0\n",
    )
    .unwrap();

    let spec_path = pkg_root.path().join("local-author.trellis.yaml");
    fs::write(
        &spec_path,
        r#"
schema_version: "0.2"
name: local-author
version: 0.1.0
description: local author flow package
homepage: https://example.org/local-author
kind: binary
source:
  type: local_dir
  path: payload
install:
  strategy: copy
  entries:
    - bin
bin:
  local-author: bin/local-author
dependencies: []
provenance:
  publisher: Local Author
  license: MIT
  registry: local-dev
platform:
  os: [linux, macos, windows]
  arch: [x86_64, aarch64]
"#,
    )
    .unwrap();

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
        .arg("install")
        .arg("--from")
        .arg(&spec_path)
        .assert()
        .success()
        .stdout(contains("Installed local-author 0.1.0"));

    assert!(home.path().join("receipts/local-author.json").exists());
}

#[test]
fn install_rejects_platform_mismatch() {
    let home = tempdir().unwrap();
    let pkg_root = tempdir().unwrap();

    fs::create_dir_all(pkg_root.path().join("payload/bin")).unwrap();
    fs::write(
        pkg_root.path().join("payload/bin/mismatch"),
        "#!/usr/bin/env sh\necho mismatch\n",
    )
    .unwrap();

    let mismatch_os = match std::env::consts::OS {
        "linux" => "macos",
        _ => "linux",
    };

    let spec_path = pkg_root.path().join("mismatch.trellis.yaml");
    let contents = format!(
        "schema_version: \"0.2\"\nname: mismatch\nversion: 0.1.0\ndescription: platform mismatch package\nhomepage: https://example.org/mismatch\nkind: binary\nsource:\n  type: local_dir\n  path: payload\ninstall:\n  strategy: copy\n  entries: [bin]\nbin:\n  mismatch: bin/mismatch\ndependencies: []\nprovenance:\n  publisher: Local\n  license: MIT\n  registry: local-dev\nplatform:\n  os: [{}]\n  arch: [{}]\n",
        mismatch_os,
        std::env::consts::ARCH
    );
    fs::write(&spec_path, contents).unwrap();

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
        .arg("install")
        .arg("--from")
        .arg(&spec_path)
        .assert()
        .failure()
        .stderr(contains("does not support this platform"));
}
