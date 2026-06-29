//! End-to-end CLI behavior tests driving the real `derust` binary.

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

const FIXTURE: &str = "tests/fixtures/single-package";

fn derust() -> Command {
    Command::cargo_bin("derust").expect("derust binary builds")
}

#[test]
fn help_lists_subcommands() {
    derust()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("scan"))
        .stdout(contains("config"));
}

#[test]
fn scan_help_lists_flags() {
    derust()
        .args(["scan", "--help"])
        .assert()
        .success()
        .stdout(contains("--json"))
        .stdout(contains("--path"));
}

#[test]
fn scan_prints_normalized_report() {
    derust()
        .args(["scan", "--path", FIXTURE])
        .assert()
        .success()
        .stdout(contains("derust scan:"))
        .stdout(contains("summary:"));
}

#[test]
fn scan_json_parses_as_per_check_array() {
    let output = derust()
        .args(["scan", "--path", FIXTURE, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 stdout");
    let value: serde_json::Value = serde_json::from_str(&text).expect("stdout is valid JSON");
    let checks = value["checks"].as_array().expect("checks is an array");
    assert_eq!(checks.len(), 4);
}

#[test]
fn scan_leaves_target_project_unmodified() {
    // derust must never modify a target project: after a scan that runs real
    // cargo checks, the fixture must not gain a Cargo.lock or a target/ dir.
    derust()
        .args(["scan", "--path", FIXTURE])
        .assert()
        .success();

    let lockfile = std::path::Path::new(FIXTURE).join("Cargo.lock");
    let target = std::path::Path::new(FIXTURE).join("target");
    assert!(
        !lockfile.exists(),
        "scan left a Cargo.lock in the target project"
    );
    assert!(
        !target.exists(),
        "scan left a target/ dir in the target project"
    );
}

#[test]
fn config_prints_effective_config() {
    derust()
        .args(["config", "--path", FIXTURE])
        .assert()
        .success()
        .stdout(contains("checks.clippy:").and(contains("false")))
        .stdout(contains("checks.check:").and(contains("true")));
}

#[test]
fn nonexistent_path_exits_nonzero_with_error() {
    derust()
        .args(["scan", "--path", "/derust/definitely/not/here"])
        .assert()
        .failure()
        .stderr(contains("no Cargo.toml found"));
}
