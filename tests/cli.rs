//! End-to-end CLI behavior tests driving the real `cargo-target-gc` binary.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

const FIXTURE: &str = "tests/fixtures/single-package";
const TEST_SECONDS_PER_DAY: u64 = 86_400;

fn target_gc() -> Command {
    Command::cargo_bin("cargo-target-gc").expect("cargo-target-gc binary builds")
}

/// Create a unique temp project with a populated `target/` tree.
///
/// `target/` contains a fresh (retained) debug profile, an old incremental subtree,
/// and an old (stale) release profile, with deterministic mtimes. Returns the
/// project root; the caller removes it.
fn temp_project(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let root = std::env::temp_dir().join(format!(
        "target-gc-cli-{tag}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create project");
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"x\"\n").expect("manifest");
    let target = root.join("target");
    fs::create_dir_all(&target).expect("target");
    fs::write(target.join("CACHEDIR.TAG"), "Signature").expect("tag");
    write_aged(&target.join("debug/deps/lib.rlib"), 1000, 0);
    write_aged(&target.join("debug/incremental/seg/x.o"), 500, 2);
    write_aged(&target.join("release/deps/lib.rlib"), 2000, 100);
    root
}

/// Write a file of `len` bytes with an mtime `age_days` in the past.
fn write_aged(path: &Path, len: usize, age_days: u64) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parents");
    }
    fs::write(path, vec![b'x'; len]).expect("write");
    let when = SystemTime::now()
        .checked_sub(Duration::from_secs(age_days * TEST_SECONDS_PER_DAY))
        .expect("aged");
    File::options()
        .write(true)
        .open(path)
        .expect("open")
        .set_modified(when)
        .expect("mtime");
}

/// Total bytes and entry count under a directory tree (for unchanged checks).
fn tree_fingerprint(dir: &Path) -> (u64, usize) {
    let mut bytes = 0u64;
    let mut count = 0usize;
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return (bytes, count),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        count += 1;
        let meta = entry.metadata().expect("metadata");
        if meta.is_dir() {
            let (b, c) = tree_fingerprint(&path);
            bytes += b;
            count += c;
        } else {
            bytes += meta.len();
        }
    }
    (bytes, count)
}

#[test]
fn help_lists_subcommands() {
    target_gc()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("scan"))
        .stdout(contains("clean"))
        .stdout(contains("config"))
        .stdout(contains("install-agent-skills"))
        .stdout(contains("same directory where you would run `cargo build`"));
}

#[test]
fn scan_help_lists_flags() {
    target_gc()
        .args(["scan", "--help"])
        .assert()
        .success()
        .stdout(contains("--json"))
        .stdout(contains("--path"))
        .stdout(contains("same directory where `cargo build`"));
}

#[test]
fn scan_reports_target_roots_and_reclaimable() {
    let root = temp_project("scan");
    target_gc()
        .args(["scan", "--path"])
        .arg(&root)
        .assert()
        .success()
        .stdout(contains("target:"))
        .stdout(contains("old incremental:"))
        .stdout(contains("fresh incremental:"))
        .stdout(contains("reclaimable:"))
        .stdout(contains("summary:"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cargo_subcommand_invocation_shape_works() {
    let root = temp_project("cargo-subcommand");
    target_gc()
        .args(["target-gc", "scan", "--path"])
        .arg(&root)
        .assert()
        .success()
        .stdout(contains("cargo target-gc scan:"))
        .stdout(contains("reclaimable:"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn scan_json_has_roots_and_reclaimable_keys() {
    let root = temp_project("scanjson");
    let output = target_gc()
        .args(["scan", "--json", "--path"])
        .arg(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 stdout");
    let value: serde_json::Value = serde_json::from_str(&text).expect("stdout is valid JSON");
    let roots = value["roots"].as_array().expect("roots is an array");
    assert_eq!(roots.len(), 1);
    assert!(roots[0]["reclaimable_bytes"].is_u64());
    assert!(roots[0]["fresh_incremental_bytes"].is_u64());
    assert!(value["summary"]["reclaimable_bytes"].is_u64());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn scan_never_runs_cargo_and_leaves_target_unchanged() {
    let root = temp_project("readonly");
    let target = root.join("target");
    let before = tree_fingerprint(&target);

    target_gc()
        .args(["scan", "--path"])
        .arg(&root)
        .assert()
        .success();

    // scan must not invoke cargo: no Cargo.lock and no extra target dir appear,
    // and the existing target tree is byte-identical.
    assert!(
        !root.join("Cargo.lock").exists(),
        "scan created a Cargo.lock"
    );
    assert_eq!(tree_fingerprint(&target), before, "scan mutated target/");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn scan_on_project_without_target_succeeds() {
    target_gc()
        .args(["scan", "--path", FIXTURE])
        .assert()
        .success()
        .stdout(contains("cargo target-gc scan:"))
        .stdout(contains("same directory where you run `cargo build`"));
    // The read-only fixture must gain no Cargo.lock or target/.
    assert!(!Path::new(FIXTURE).join("Cargo.lock").exists());
    assert!(!Path::new(FIXTURE).join("target").exists());
}

#[test]
fn clean_help_lists_flags() {
    target_gc()
        .args(["clean", "--help"])
        .assert()
        .success()
        .stdout(contains("--dry-run"))
        .stdout(contains("--confirm"))
        .stdout(contains("--force-active"))
        .stdout(contains("--max-reclaim"))
        .stdout(contains("--stale"));
}

#[test]
fn install_agent_skills_help_lists_host_flags() {
    target_gc()
        .args(["install-agent-skills", "--help"])
        .assert()
        .success()
        .stdout(contains("--claude-skills-dir"))
        .stdout(contains("--codex-skills-dir"))
        .stdout(contains("--only"))
        .stdout(contains("--all"))
        .stdout(contains("--dry-run"))
        .stdout(contains("--yes"));
}

#[test]
fn install_agent_skills_dry_run_writes_nothing() {
    let root = temp_project("skill-dry-run");
    let claude = root.join("claude-skills");

    target_gc()
        .args([
            "install-agent-skills",
            "--only",
            "claude",
            "--dry-run",
            "--claude-skills-dir",
        ])
        .arg(&claude)
        .assert()
        .success()
        .stdout(contains("Would install Claude Code skill"));

    assert!(!claude.join("cargo-target-gc/SKILL.md").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn install_agent_skills_writes_selected_hosts() {
    let root = temp_project("skill-install");
    let claude = root.join("claude-skills");
    let codex = root.join("codex-skills");

    target_gc()
        .args([
            "install-agent-skills",
            "--only",
            "claude,codex",
            "--claude-skills-dir",
        ])
        .arg(&claude)
        .args(["--codex-skills-dir"])
        .arg(&codex)
        .assert()
        .success()
        .stdout(contains("Installed Claude Code skill"))
        .stdout(contains("Installed Codex skill"));

    let claude_skill =
        fs::read_to_string(claude.join("cargo-target-gc/SKILL.md")).expect("claude skill");
    let codex_skill =
        fs::read_to_string(codex.join("cargo-target-gc/SKILL.md")).expect("codex skill");
    assert!(claude_skill.contains("name: cargo-target-gc"));
    assert!(claude_skill.contains("cargo target-gc scan"));
    assert!(claude_skill.contains("Do not run `cargo target-gc clean --confirm`"));
    assert_eq!(claude_skill, codex_skill);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn install_agent_skills_skip_existing_preserves_file() {
    let root = temp_project("skill-skip-existing");
    let claude = root.join("claude-skills");
    let skill = claude.join("cargo-target-gc/SKILL.md");
    fs::create_dir_all(skill.parent().expect("skill parent")).expect("parent");
    fs::write(&skill, "custom skill").expect("write existing skill");

    target_gc()
        .args([
            "install-agent-skills",
            "--only",
            "claude",
            "--skip-existing",
            "--claude-skills-dir",
        ])
        .arg(&claude)
        .assert()
        .success()
        .stdout(contains("Keeping existing Claude Code skill"));

    assert_eq!(fs::read_to_string(skill).expect("skill"), "custom skill");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cargo_subcommand_can_install_agent_skills() {
    let root = temp_project("skill-cargo-subcommand");
    let codex = root.join("codex-skills");

    target_gc()
        .args([
            "target-gc",
            "install-agent-skills",
            "--only",
            "codex",
            "--codex-skills-dir",
        ])
        .arg(&codex)
        .assert()
        .success()
        .stdout(contains("Installed Codex skill"));

    assert!(codex.join("cargo-target-gc/SKILL.md").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn clean_without_mode_refuses() {
    let root = temp_project("refuse");
    target_gc()
        .args(["clean", "--path"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(contains("--dry-run").or(contains("--confirm")));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn clean_rejects_both_dry_run_and_confirm() {
    let root = temp_project("conflict");
    let target = root.join("target");
    let before = tree_fingerprint(&target);

    target_gc()
        .args(["clean", "--dry-run", "--confirm", "--path"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));

    // A rejected invocation must never touch the target tree.
    assert_eq!(
        tree_fingerprint(&target),
        before,
        "conflicting flags mutated target/"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn clean_dry_run_leaves_target_unchanged() {
    let root = temp_project("dryrun");
    let target = root.join("target");
    let before = tree_fingerprint(&target);

    target_gc()
        .args(["clean", "--dry-run", "--stale", "--path"])
        .arg(&root)
        .assert()
        .success()
        .stdout(contains("would remove"))
        .stdout(contains("reclaimable:"));

    assert_eq!(tree_fingerprint(&target), before, "dry-run mutated target/");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn clean_confirm_removes_reclaimable_preserves_retained() {
    let root = temp_project("confirm");
    let target = root.join("target");

    target_gc()
        .args(["clean", "--confirm", "--stale", "--path"])
        .arg(&root)
        .assert()
        .success()
        .stdout(contains("removed"))
        .stdout(contains("reclaimed:"));

    // Reclaimable artifacts are gone; retained + tag survive.
    assert!(!target.join("debug/incremental").exists());
    assert!(!target.join("release/deps/lib.rlib").exists());
    assert!(target.join("debug/deps/lib.rlib").exists());
    assert!(target.join("CACHEDIR.TAG").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn clean_confirm_refuses_over_max_reclaim() {
    let root = temp_project("maxreclaim");
    let target = root.join("target");
    let before = tree_fingerprint(&target);

    target_gc()
        .args([
            "clean",
            "--confirm",
            "--stale",
            "--max-reclaim",
            "100B",
            "--path",
        ])
        .arg(&root)
        .assert()
        .failure()
        .stderr(contains("exceeds the limit"));

    assert_eq!(
        tree_fingerprint(&target),
        before,
        "max-reclaim refusal mutated target/"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn config_prints_retention_days() {
    target_gc()
        .args(["config", "--path", FIXTURE])
        .assert()
        .success()
        .stdout(contains("retention_days:").and(contains("30")))
        .stdout(contains("incremental_retention_hours:").and(contains("12")))
        .stdout(contains("max_reclaim_bytes:").and(contains("1048576")));
}

#[test]
fn nonexistent_path_exits_nonzero_with_error() {
    target_gc()
        .args(["scan", "--path", "/cargo-target-gc/definitely/not/here"])
        .assert()
        .failure()
        .stderr(contains("no Cargo.toml found"));
}
