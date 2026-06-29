//! Scan orchestration: discovery → config → probe → report.
//!
//! Strictly read-only: it discovers the project, reads config, and runs the
//! read-only cargo probes, then assembles a [`ScanReport`].

use std::fmt;
use std::path::{Path, PathBuf};

use crate::config::{self, Checks, ConfigError};
use crate::discovery::{self, DiscoveryError, Project, ProjectKind};
use crate::probe::{self, Check, CheckResult, CheckStatus};
use crate::report::{CheckEntry, ScanReport, Summary};

/// Errors that can abort a scan before any report is produced.
#[derive(Debug)]
pub enum ScanError {
    /// Project discovery failed.
    Discovery(DiscoveryError),
    /// Configuration loading failed.
    Config(ConfigError),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::Discovery(e) => write!(f, "{e}"),
            ScanError::Config(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ScanError {}

impl From<DiscoveryError> for ScanError {
    fn from(e: DiscoveryError) -> Self {
        ScanError::Discovery(e)
    }
}

impl From<ConfigError> for ScanError {
    fn from(e: ConfigError) -> Self {
        ScanError::Config(e)
    }
}

/// Run a read-only scan of the project rooted at or above `path`.
pub fn scan(path: &Path) -> Result<ScanReport, ScanError> {
    let project = discovery::discover(path)?;
    let cfg = config::load(&project.root)?;
    let target = target_dir(&project.root, &cfg.crate_path);
    let cargo = probe::cargo_available();

    // Guard the project against cargo's writes (target artifacts + lockfile) for
    // the whole scan; dropped after the loop, which restores the project.
    let sandbox = probe::Sandbox::new(&project.root);

    let mut summary = Summary::default();
    let mut checks = Vec::with_capacity(Check::ALL.len());
    for check in Check::ALL {
        let entry = evaluate(check, &target, sandbox.target_dir(), &cfg.checks, cargo);
        summary.record(entry.status);
        checks.push(entry);
    }

    Ok(build_report(&project, checks, summary))
}

/// Resolve the directory checks run in, honoring an optional crate path.
fn target_dir(root: &Path, crate_path: &Option<PathBuf>) -> PathBuf {
    match crate_path {
        Some(p) => root.join(p),
        None => root.to_path_buf(),
    }
}

/// Whether `check` is enabled by the configured toggles.
fn is_enabled(check: Check, checks: &Checks) -> bool {
    match check {
        Check::Check => checks.check,
        Check::Test => checks.test,
        Check::Fmt => checks.fmt,
        Check::Clippy => checks.clippy,
    }
}

/// Run (or skip) a single check and turn it into a report entry.
///
/// `out_dir` is the sandbox's isolated `CARGO_TARGET_DIR` so the check never
/// writes build artifacts into the target project.
fn evaluate(check: Check, dir: &Path, out_dir: &Path, checks: &Checks, cargo: bool) -> CheckEntry {
    if !is_enabled(check, checks) {
        return CheckEntry {
            name: check.name().into(),
            status: CheckStatus::Skipped,
            detail: None,
        };
    }
    if !cargo {
        return CheckEntry {
            name: check.name().into(),
            status: CheckStatus::Unavailable,
            detail: Some("cargo not available".into()),
        };
    }
    let result = probe::run_check(dir, check.args(), out_dir);
    CheckEntry {
        name: check.name().into(),
        status: result.status,
        detail: detail_for(&result),
    }
}

/// Extract a short detail line for non-ok results.
fn detail_for(result: &CheckResult) -> Option<String> {
    if result.status == CheckStatus::Ok {
        return None;
    }
    let source = if result.stderr.trim().is_empty() {
        &result.stdout
    } else {
        &result.stderr
    };
    source
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

/// Assemble the final report from project metadata and check entries.
fn build_report(project: &Project, checks: Vec<CheckEntry>, summary: Summary) -> ScanReport {
    let kind = match project.kind {
        ProjectKind::Package => "package",
        ProjectKind::Workspace => "workspace",
    };
    ScanReport {
        root: project.root.display().to_string(),
        kind: kind.into(),
        crates: project.crates.len(),
        checks,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_check_is_skipped() {
        let checks = Checks {
            clippy: false,
            ..Checks::default()
        };
        let entry = evaluate(Check::Clippy, Path::new("."), Path::new("."), &checks, true);
        assert_eq!(entry.status, CheckStatus::Skipped);
    }

    #[test]
    fn missing_cargo_is_unavailable() {
        let entry = evaluate(
            Check::Check,
            Path::new("."),
            Path::new("."),
            &Checks::default(),
            false,
        );
        assert_eq!(entry.status, CheckStatus::Unavailable);
    }

    #[test]
    fn target_dir_honors_crate_path() {
        let dir = target_dir(Path::new("/root"), &Some(PathBuf::from("crates/core")));
        assert_eq!(dir, PathBuf::from("/root/crates/core"));
    }
}
