//! Active Cargo build detection for safe cleanup.
//!
//! This is a conservative process-table guard. It catches the common dangerous
//! case where `rustc`/cargo tooling is actively using the selected `target/`
//! path. False negatives are possible on platforms with limited process
//! visibility, so this guard complements rather than replaces path validation.

use std::fmt;
use std::path::PathBuf;
use std::process::Command;

const BUILD_TOOLS: [&str; 4] = ["cargo-watch", "cargo watch", "cargo", "rustc"];

/// One process that appears to be using a target root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveBuild {
    /// Process id as reported by `ps`.
    pub pid: String,
    /// Target root matched by the command line.
    pub root: PathBuf,
    /// Full command line.
    pub command: String,
}

impl fmt::Display for ActiveBuild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pid {} appears active for {}: {}",
            self.pid,
            self.root.display(),
            self.command
        )
    }
}

/// Detect active cargo/rustc/cargo-watch processes for `roots`.
pub fn detect(roots: &[PathBuf]) -> Vec<ActiveBuild> {
    let output = Command::new("ps").args(["-axo", "pid=,command="]).output();
    let Ok(output) = output else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    detect_in_process_table(&text, roots)
}

/// Parse `ps` output and return active build matches.
fn detect_in_process_table(processes: &str, roots: &[PathBuf]) -> Vec<ActiveBuild> {
    let root_keys: Vec<(PathBuf, String)> = roots
        .iter()
        .map(|root| {
            let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| root.clone());
            (root.clone(), canonical.display().to_string())
        })
        .collect();

    let mut matches = Vec::new();
    for line in processes.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !mentions_build_tool(trimmed) {
            continue;
        }
        let (pid, command) = split_pid_command(trimmed);
        for (root, key) in &root_keys {
            if command.contains(key) {
                matches.push(ActiveBuild {
                    pid: pid.to_string(),
                    root: root.clone(),
                    command: command.to_string(),
                });
                break;
            }
        }
    }
    matches
}

fn split_pid_command(line: &str) -> (&str, &str) {
    line.split_once(char::is_whitespace)
        .map(|(pid, command)| (pid, command.trim()))
        .unwrap_or((line, ""))
}

fn mentions_build_tool(command: &str) -> bool {
    BUILD_TOOLS.iter().any(|tool| command.contains(tool))
}

/// Public test hook for process-table parsing.
#[cfg(test)]
pub(crate) fn detect_in_text(processes: &str, roots: &[PathBuf]) -> Vec<ActiveBuild> {
    detect_in_process_table(processes, roots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rustc_using_target_root() {
        let root = PathBuf::from("/tmp/project/target");
        let ps = "123 /toolchain/bin/rustc --out-dir /tmp/project/target/debug/deps\n";
        let matches = detect_in_text(ps, &[root.clone()]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pid, "123");
        assert_eq!(matches[0].root, root);
    }

    #[test]
    fn ignores_unrelated_cargo_process() {
        let root = PathBuf::from("/tmp/project/target");
        let ps = "123 cargo build --manifest-path /other/Cargo.toml\n";
        assert!(detect_in_text(ps, &[root]).is_empty());
    }
}
