use assert_cmd::Command;
use predicates::str::contains;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use walkdir::WalkDir;

fn sha256_dir(path: &Path) -> String {
    let mut files = Vec::new();
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();

    let mut hasher = Sha256::new();
    for file in files {
        let rel = file
            .strip_prefix(path)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        hasher.update(rel.as_bytes());
        hasher.update([0]);
        hasher.update(fs::read(&file).unwrap());
        hasher.update([0xff]);
    }
    format!("{:x}", hasher.finalize())
}

fn write_package(root: &Path, name: &str, checksum: Option<String>) {
    fs::create_dir_all(root.join(name).join("payload/bin")).unwrap();
    fs::write(
        root.join(name).join("payload/bin").join(name),
        format!("#!/usr/bin/env sh\necho {}\n", name),
    )
    .unwrap();

    let checksum_line = checksum
        .map(|c| format!("  checksum_sha256: {}\n", c))
        .unwrap_or_default();
    let spec = format!(
        "schema_version: \"0.2\"\nname: {name}\nversion: 0.1.0\ndescription: {name}\nhomepage: https://example.org/{name}\nkind: binary\nsource:\n  type: local_dir\n  path: payload\n{checksum_line}  signature: sig:test-{name}\ninstall:\n  strategy: copy\n  entries: [bin]\nbin:\n  {name}: bin/{name}\ndependencies: []\nprovenance:\n  publisher: test\n  license: MIT\n  registry: vineyard-core\n"
    );
    fs::write(root.join(name).join(format!("{}.trellis.yaml", name)), spec).unwrap();
}

#[test]
fn install_verifies_checksum_and_writes_trust_receipt_fields() {
    let home = tempdir().unwrap();
    let registry = tempdir().unwrap();

    write_package(registry.path(), "alpha", None);
    let digest = sha256_dir(&registry.path().join("alpha/payload"));
    let spec_path = registry.path().join("alpha/alpha.trellis.yaml");
    let mut spec = fs::read_to_string(&spec_path).unwrap();
    spec = spec.replace(
        "signature: sig:test-alpha",
        &format!("checksum_sha256: {}\n  signature: sig:test-alpha", digest),
    );
    fs::write(&spec_path, spec).unwrap();

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
        .args(["install", "alpha"])
        .assert()
        .success()
        .stdout(contains("Checksum: Verified"));

    let receipt: Value =
        serde_json::from_str(&fs::read_to_string(home.path().join("receipts/alpha.json")).unwrap())
            .unwrap();
    assert_eq!(receipt["schema_version"], "0.4");
    assert_eq!(receipt["trust"]["checksum_state"], "verified");
    assert_eq!(receipt["provenance"]["signature"]["state"], "present");
    assert!(receipt["transaction_id"]
        .as_str()
        .unwrap()
        .starts_with("install-alpha"));
}

#[test]
fn install_fails_on_checksum_mismatch_and_binary_collision() {
    let home = tempdir().unwrap();
    let registry = tempdir().unwrap();

    write_package(
        registry.path(),
        "alpha2",
        Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
    );
    write_package(registry.path(), "beta2", None);

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
        .args(["install", "alpha2"])
        .assert()
        .failure()
        .stderr(contains("checksum mismatch"));

    // force a binary collision by reusing exposed command name
    let bspec = registry.path().join("beta2/beta2.trellis.yaml");
    let btxt = fs::read_to_string(&bspec)
        .unwrap()
        .replace("beta2: bin/beta2", "alpha2: bin/beta2");
    fs::write(&bspec, btxt).unwrap();

    let aspec = registry.path().join("alpha2/alpha2.trellis.yaml");
    let adigest = sha256_dir(&registry.path().join("alpha2/payload"));
    let atxt = fs::read_to_string(&aspec).unwrap().replace(
        "0000000000000000000000000000000000000000000000000000000000000000",
        &adigest,
    );
    fs::write(&aspec, atxt).unwrap();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(registry.path())
        .args(["update"])
        .assert()
        .success();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(registry.path())
        .args(["install", "alpha2"])
        .assert()
        .success();

    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--home")
        .arg(home.path())
        .arg("--registry-root")
        .arg(registry.path())
        .args(["install", "beta2"])
        .assert()
        .failure()
        .stderr(contains("binary 'alpha2' is already managed"));
}
