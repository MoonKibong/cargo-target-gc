//! Toolchain probing and the shared, read-only cargo command runner.
//!
//! Every check funnels through [`run_check`], so the four checks
//! (check / test / fmt / clippy) share one execution path with no copy-paste.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// The outcome of a single check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    /// The check ran and passed.
    Ok,
    /// The check ran and failed.
    Failed,
    /// The check was disabled by configuration.
    Skipped,
    /// The check could not run (tool not installed / not invokable).
    Unavailable,
}

impl fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            CheckStatus::Ok => "ok",
            CheckStatus::Failed => "failed",
            CheckStatus::Skipped => "skipped",
            CheckStatus::Unavailable => "unavailable",
        };
        f.write_str(label)
    }
}

/// The identity of a check, with the read-only cargo args it runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    Check,
    Test,
    Fmt,
    Clippy,
}

impl Check {
    /// All checks in canonical reporting order.
    pub const ALL: [Check; 4] = [Check::Check, Check::Test, Check::Fmt, Check::Clippy];

    /// Stable lowercase name used in reports and JSON.
    pub fn name(&self) -> &'static str {
        match self {
            Check::Check => "check",
            Check::Test => "test",
            Check::Fmt => "fmt",
            Check::Clippy => "clippy",
        }
    }

    /// Read-only cargo arguments for this check.
    pub fn args(&self) -> &'static [&'static str] {
        match self {
            Check::Check => &["check"],
            Check::Test => &["test"],
            Check::Fmt => &["fmt", "--check"],
            Check::Clippy => &["clippy", "--", "-D", "warnings"],
        }
    }
}

/// The captured result of running a check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub status: CheckStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Substrings cargo/rustup emit when a subcommand or toolchain component is
/// not installed, as opposed to when a check genuinely fails. `cargo fmt` and
/// `cargo clippy` are optional components (rustfmt / clippy); when they are
/// absent cargo exits nonzero with one of these markers, which the MVP must
/// treat as "unavailable" rather than "failed".
fn stderr_signals_unavailable(stderr: &str) -> bool {
    const MARKERS: [&str; 3] = [
        // cargo, when the subcommand binary (cargo-fmt / cargo-clippy) is absent
        "no such command",
        // rustup, e.g. "'rustfmt' is not installed for the toolchain"
        "is not installed",
        // rustup, e.g. "component 'clippy' ... is not available"
        "is not available",
    ];
    MARKERS.iter().any(|m| stderr.contains(m))
}

/// Map a process exit code and its stderr into a [`CheckStatus`].
///
/// A nonzero exit usually means the check failed, but a missing optional
/// component (rustfmt / clippy) also exits nonzero; those are surfaced as
/// [`CheckStatus::Unavailable`] via [`stderr_signals_unavailable`].
fn status_from_output(code: Option<i32>, stderr: &str) -> CheckStatus {
    match code {
        Some(0) => CheckStatus::Ok,
        Some(_) if stderr_signals_unavailable(stderr) => CheckStatus::Unavailable,
        Some(_) => CheckStatus::Failed,
        None => CheckStatus::Unavailable,
    }
}

/// Build a [`CheckResult`] from captured process output.
fn result_from_output(output: Output) -> CheckResult {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    CheckResult {
        status: status_from_output(output.status.code(), &stderr),
        stdout,
        stderr,
    }
}

/// A read-only execution sandbox for cargo checks.
///
/// Even in `check` / `test` / `--check` modes, cargo writes build artifacts to
/// `target/` and creates or updates `Cargo.lock`. derust must never modify a
/// target project, so the sandbox makes the runs non-destructive by:
///
/// 1. redirecting all build output to an isolated temp directory via
///    `CARGO_TARGET_DIR`, so no `target/` lands in the project; and
/// 2. snapshotting the project lockfile up front and restoring it on drop, so a
///    `Cargo.lock` cargo writes during the scan is reverted and the project is
///    left byte-identical.
///
/// The sandbox owns the temp target dir for the lifetime of a scan; dropping it
/// removes the temp dir and restores the lockfile even if a check panics.
pub struct Sandbox {
    /// Isolated `CARGO_TARGET_DIR` for the duration of the scan.
    target_dir: PathBuf,
    /// Project lockfile guarded against creation/modification.
    lockfile: PathBuf,
    /// Lockfile contents before the scan, or `None` if it did not exist.
    lock_before: Option<Vec<u8>>,
}

impl Sandbox {
    /// Open a sandbox guarding the `Cargo.lock` at `project_root`.
    pub fn new(project_root: &Path) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let target_dir =
            std::env::temp_dir().join(format!("derust-target-{}-{}", std::process::id(), nanos));
        let lockfile = project_root.join("Cargo.lock");
        let lock_before = fs::read(&lockfile).ok();
        Sandbox {
            target_dir,
            lockfile,
            lock_before,
        }
    }

    /// The isolated build directory checks should target.
    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // Restore the lockfile to its pre-scan state (revert, or remove if cargo
        // created it), then discard the isolated build artifacts. Errors are
        // best-effort: a read-only scan must not abort cleanup.
        match &self.lock_before {
            Some(bytes) => {
                let _ = fs::write(&self.lockfile, bytes);
            }
            None => {
                let _ = fs::remove_file(&self.lockfile);
            }
        }
        let _ = fs::remove_dir_all(&self.target_dir);
    }
}

/// Run a read-only `cargo` invocation in `dir`, sending build output to the
/// isolated `target_dir`, and capturing its result.
///
/// A spawn failure (cargo missing) maps to [`CheckStatus::Unavailable`].
pub fn run_check(dir: &Path, args: &[&str], target_dir: &Path) -> CheckResult {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .env("CARGO_TARGET_DIR", target_dir)
        .output();
    match output {
        Ok(output) => result_from_output(output),
        Err(e) => CheckResult {
            status: CheckStatus::Unavailable,
            stdout: String::new(),
            stderr: e.to_string(),
        },
    }
}

/// Report whether `cargo` itself is invokable on this machine.
pub fn cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    fn output(code: i32, stderr: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(code << 8),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn zero_code_is_ok() {
        assert_eq!(result_from_output(output(0, "")).status, CheckStatus::Ok);
    }

    #[test]
    fn nonzero_code_is_failed() {
        assert_eq!(
            result_from_output(output(1, "error: build failed")).status,
            CheckStatus::Failed
        );
    }

    #[test]
    fn missing_code_is_unavailable() {
        assert_eq!(status_from_output(None, ""), CheckStatus::Unavailable);
    }

    #[test]
    fn missing_subcommand_is_unavailable() {
        assert_eq!(
            status_from_output(Some(101), "error: no such command: `clippy`"),
            CheckStatus::Unavailable
        );
    }

    #[test]
    fn missing_component_is_unavailable() {
        assert_eq!(
            status_from_output(
                Some(1),
                "error: 'rustfmt' is not installed for the toolchain"
            ),
            CheckStatus::Unavailable
        );
    }

    #[test]
    fn check_args_are_read_only() {
        assert_eq!(Check::Fmt.args(), &["fmt", "--check"]);
        assert_eq!(Check::Clippy.args(), &["clippy", "--", "-D", "warnings"]);
    }
}
