//! Scan report data model and rendering (human-readable + JSON).

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::probe::CheckStatus;

/// One check's entry in a scan report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckEntry {
    /// Check name (e.g. `clippy`).
    pub name: String,
    /// Outcome of the check.
    pub status: CheckStatus,
    /// Short detail (e.g. first stderr line) when not `Ok`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Aggregate counts across all checks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub ok: usize,
    pub failed: usize,
    pub skipped: usize,
    pub unavailable: usize,
}

impl Summary {
    /// Tally a status into the summary.
    pub fn record(&mut self, status: CheckStatus) {
        match status {
            CheckStatus::Ok => self.ok += 1,
            CheckStatus::Failed => self.failed += 1,
            CheckStatus::Skipped => self.skipped += 1,
            CheckStatus::Unavailable => self.unavailable += 1,
        }
    }
}

/// The full result of a read-only project scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanReport {
    /// Absolute or relative root path that was scanned.
    pub root: String,
    /// `package` or `workspace`.
    pub kind: String,
    /// Number of crate manifests discovered.
    pub crates: usize,
    /// Per-check results in canonical order.
    pub checks: Vec<CheckEntry>,
    /// Aggregate counts.
    pub summary: Summary,
}

impl ScanReport {
    /// Render a normalized, human-readable summary.
    pub fn render_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "derust scan: {}", self.root);
        let _ = writeln!(out, "  project: {} ({} crate(s))", self.kind, self.crates);
        let _ = writeln!(out, "  checks:");
        for entry in &self.checks {
            match &entry.detail {
                Some(detail) => {
                    let _ = writeln!(out, "    {:<8} {}  — {}", entry.name, entry.status, detail);
                }
                None => {
                    let _ = writeln!(out, "    {:<8} {}", entry.name, entry.status);
                }
            }
        }
        let s = &self.summary;
        let _ = writeln!(
            out,
            "  summary: {} ok, {} failed, {} skipped, {} unavailable",
            s.ok, s.failed, s.skipped, s.unavailable
        );
        out
    }

    /// Render the report as pretty JSON.
    pub fn render_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ScanReport {
        let mut summary = Summary::default();
        summary.record(CheckStatus::Ok);
        summary.record(CheckStatus::Failed);
        ScanReport {
            root: "tests/fixtures/single-package".into(),
            kind: "package".into(),
            crates: 1,
            checks: vec![
                CheckEntry {
                    name: "check".into(),
                    status: CheckStatus::Ok,
                    detail: None,
                },
                CheckEntry {
                    name: "clippy".into(),
                    status: CheckStatus::Failed,
                    detail: Some("1 warning".into()),
                },
            ],
            summary,
        }
    }

    #[test]
    fn json_round_trips() {
        let report = sample();
        let json = report.render_json().expect("serialize");
        let parsed: ScanReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(report, parsed);
    }

    #[test]
    fn text_lists_each_check() {
        let text = sample().render_text();
        assert!(text.contains("check"));
        assert!(text.contains("clippy"));
        assert!(text.contains("summary:"));
    }
}
