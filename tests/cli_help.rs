use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn top_level_help_lists_rc1_commands_and_global_flags() {
    Command::cargo_bin("trellis")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("init"))
        .stdout(contains("seed"))
        .stdout(contains("bootstrap"))
        .stdout(contains("verify"))
        .stdout(contains("repair"))
        .stdout(contains("--home"))
        .stdout(contains("--registry-root"))
        .stdout(contains("--profile"));
}

#[test]
fn install_help_is_explicit_about_name_and_from_path() {
    Command::cargo_bin("trellis")
        .unwrap()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(contains(
            "Install a package by name or directly from a spec path",
        ))
        .stdout(contains("Package name present in active registry index"))
        .stdout(contains("Install directly from a .trellis.yaml path"));
}
