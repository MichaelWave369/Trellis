use assert_cmd::Command;
use predicates::str::contains;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn update_reports_malformed_specs_but_succeeds() {
    let home = tempdir().unwrap();
    let registry = tempdir().unwrap();
    fs::create_dir_all(registry.path().join("good/payload/bin")).unwrap();
    fs::write(
        registry.path().join("good/payload/bin/good"),
        "#!/usr/bin/env sh\necho good\n",
    )
    .unwrap();
    fs::write(
        registry.path().join("good/good.trellis.yaml"),
        r#"schema_version: "0.2"
name: good
version: 0.1.0
description: good
homepage: https://example.org/good
kind: binary
source:
  type: local_dir
  path: payload
install:
  strategy: copy
  entries: [bin]
bin:
  good: bin/good
dependencies: []
provenance:
  publisher: local
  license: MIT
  registry: vineyard-core
"#,
    )
    .unwrap();
    fs::create_dir_all(registry.path().join("bad")).unwrap();
    fs::write(registry.path().join("bad/bad.trellis.yaml"), "name: bad\n").unwrap();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(registry.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(registry.path())
        .args(["update"])
        .assert()
        .success()
        .stdout(contains("1 malformed"));

    let index = fs::read_to_string(home.path().join("registry/index.json")).unwrap();
    assert!(index.contains("\"skipped\""));
    assert!(index.contains("bad.trellis.yaml"));
}

#[test]
fn doctor_fails_on_stale_or_malformed_registry_health() {
    let home = tempdir().unwrap();
    let registry_root = format!("{}/packages", env!("CARGO_MANIFEST_DIR"));

    for args in [["init"], ["update"]] {
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

    let index_path = home.path().join("registry/index.json");
    let mut index: Value = serde_json::from_str(&fs::read_to_string(&index_path).unwrap()).unwrap();
    index["generated_at"] = Value::String("1999-01-01T00:00:00Z".to_string());
    fs::write(&index_path, serde_json::to_string_pretty(&index).unwrap()).unwrap();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(&registry_root)
        .args(["doctor"])
        .assert()
        .failure()
        .stdout(contains("stale index"));
}
