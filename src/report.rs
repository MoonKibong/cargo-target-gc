//! Scan report data model and rendering (human-readable + JSON).
//!
//! JSON carries raw byte counts; text rendering humanizes sizes. Both describe
//! the same target-artifact analysis produced by [`crate::target`].

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

/// One target root's contribution to a scan report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetRootReport {
    /// Path to the `target/` root.
    pub path: String,
    /// Total bytes across every category.
    pub total_bytes: u64,
    /// Bytes in old `incremental/` subtrees eligible for cleanup.
    pub incremental_bytes: u64,
    /// Bytes in recent `incremental/` subtrees retained as warm cache.
    pub fresh_incremental_bytes: u64,
    /// Bytes in stale profile artifacts.
    pub stale_bytes: u64,
    /// Bytes in retained (build-hot) artifacts.
    pub retained_bytes: u64,
    /// Estimated reclaimable bytes (`incremental_bytes` + `stale_bytes`).
    pub reclaimable_bytes: u64,
}

/// Aggregate totals across all target roots.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    /// Number of target roots analyzed.
    pub roots: usize,
    /// Total bytes across all roots.
    pub total_bytes: u64,
    /// Total reclaimable bytes across all roots.
    pub reclaimable_bytes: u64,
}

/// The full result of a read-only target-artifact scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanReport {
    /// Project root that was scanned.
    pub root: String,
    /// Per-target-root breakdowns.
    pub roots: Vec<TargetRootReport>,
    /// Aggregate totals.
    pub summary: Summary,
}

impl ScanReport {
    /// Render a normalized, human-readable summary with humanized sizes.
    pub fn render_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "cargo target-gc scan: {}", self.root);
        if self.roots.is_empty() {
            let _ = writeln!(out, "  no target/ directories found — nothing to reclaim");
            let _ = writeln!(
                out,
                "  run cargo target-gc from the same directory where you run `cargo build`"
            );
            let _ = writeln!(
                out,
                "  if a wrapper such as `make` builds a nested Cargo project, cd there first"
            );
            return out;
        }
        for root in &self.roots {
            let _ = writeln!(out, "  target: {}", root.path);
            let _ = writeln!(out, "    total:       {}", human(root.total_bytes));
            let _ = writeln!(
                out,
                "    old incremental:   {}  reclaimable",
                human(root.incremental_bytes)
            );
            let _ = writeln!(
                out,
                "    fresh incremental: {}  retained for edit-build speed",
                human(root.fresh_incremental_bytes)
            );
            let _ = writeln!(out, "    stale:       {}", human(root.stale_bytes));
            let _ = writeln!(out, "    retained:    {}", human(root.retained_bytes));
            let _ = writeln!(out, "    reclaimable: {}", human(root.reclaimable_bytes));
        }
        let s = &self.summary;
        let _ = writeln!(
            out,
            "  summary: {} root(s), {} total, {} reclaimable",
            s.roots,
            human(s.total_bytes),
            human(s.reclaimable_bytes)
        );
        out
    }

    /// Render the report as pretty JSON (raw byte counts).
    pub fn render_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Format a byte count into a human-readable size (binary units).
pub fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ScanReport {
        ScanReport {
            root: "/proj".into(),
            roots: vec![TargetRootReport {
                path: "/proj/target".into(),
                total_bytes: 3_250_000,
                incremental_bytes: 1_000_000,
                fresh_incremental_bytes: 250_000,
                stale_bytes: 500_000,
                retained_bytes: 1_750_000,
                reclaimable_bytes: 1_500_000,
            }],
            summary: Summary {
                roots: 1,
                total_bytes: 3_250_000,
                reclaimable_bytes: 1_500_000,
            },
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
    fn text_lists_roots_and_reclaimable() {
        let text = sample().render_text();
        assert!(text.contains("/proj/target"));
        assert!(text.contains("fresh incremental:"));
        assert!(text.contains("old incremental:"));
        assert!(text.contains("reclaimable:"));
        assert!(text.contains("summary:"));
    }

    #[test]
    fn human_uses_binary_units() {
        assert_eq!(human(512), "512 B");
        assert_eq!(human(1024), "1.0 KiB");
        assert_eq!(human(1024 * 1024), "1.0 MiB");
    }
}
