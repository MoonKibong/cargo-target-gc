//! Scan orchestration: discovery → config → read-only target analysis → report.
//!
//! `scan` is a pure filesystem analysis. It NEVER invokes cargo and creates no
//! build artifacts; it only reads `target/` directories to estimate reclaimable
//! space. Progress is written to stderr; the report is returned for stdout.

use std::fmt;
use std::path::Path;

use crate::config::{self, ConfigError};
use crate::discovery::{self, DiscoveryError};
use crate::report::{ScanReport, Summary, TargetRootReport};
use crate::target::{self, RootAnalysis, TargetError};

/// Errors that can abort a scan before any report is produced.
#[derive(Debug)]
pub enum ScanError {
    /// Project discovery failed.
    Discovery(DiscoveryError),
    /// Configuration loading failed.
    Config(ConfigError),
    /// A target root could not be analyzed.
    Target(TargetError),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::Discovery(e) => write!(f, "{e}"),
            ScanError::Config(e) => write!(f, "{e}"),
            ScanError::Target(e) => write!(f, "{e}"),
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

impl From<TargetError> for ScanError {
    fn from(e: TargetError) -> Self {
        ScanError::Target(e)
    }
}

/// Run a read-only target-artifact scan of the project rooted at or above
/// `path`. Emits progress to stderr; returns the assembled report.
pub fn scan(path: &Path) -> Result<ScanReport, ScanError> {
    let project = discovery::discover(path)?;
    let cfg = config::load(&project.root)?;
    let roots = target::locate_roots(&project, &cfg.crate_path)?;

    eprintln!(
        "cargo-target-gc: analyzing {} target root(s) under {}",
        roots.len(),
        project.root.display()
    );

    let mut analyses = Vec::with_capacity(roots.len());
    for root in &roots {
        eprintln!("cargo-target-gc: scanning {}", root.display());
        analyses.push(target::analyze(
            root,
            cfg.retention_days,
            cfg.incremental_retention_hours,
        )?);
    }

    Ok(build_report(&project.root.display().to_string(), analyses))
}

/// Assemble the final report from per-root analyses.
fn build_report(root: &str, analyses: Vec<RootAnalysis>) -> ScanReport {
    let mut summary = Summary {
        roots: analyses.len(),
        ..Summary::default()
    };
    let roots = analyses
        .iter()
        .map(|a| {
            summary.total_bytes += a.total_bytes;
            summary.reclaimable_bytes += a.reclaimable_bytes();
            TargetRootReport {
                path: a.root.display().to_string(),
                total_bytes: a.total_bytes,
                incremental_bytes: a.incremental_bytes,
                fresh_incremental_bytes: a.fresh_incremental_bytes,
                profile_cache_bytes: a.profile_cache_bytes,
                stale_bytes: a.stale_bytes,
                retained_bytes: a.retained_bytes,
                reclaimable_bytes: a.reclaimable_bytes(),
            }
        })
        .collect();
    ScanReport {
        root: root.to_string(),
        roots,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn analysis(
        root: &str,
        incremental: u64,
        fresh_incremental: u64,
        stale: u64,
        retained: u64,
    ) -> RootAnalysis {
        RootAnalysis {
            root: PathBuf::from(root),
            total_bytes: incremental + fresh_incremental + stale + retained,
            incremental_bytes: incremental,
            fresh_incremental_bytes: fresh_incremental,
            profile_cache_bytes: 0,
            stale_bytes: stale,
            retained_bytes: fresh_incremental + retained,
            artifacts: Vec::new(),
        }
    }

    #[test]
    fn report_aggregates_roots() {
        let report = build_report(
            "/proj",
            vec![
                analysis("/proj/target", 100, 25, 50, 200),
                analysis("/proj/a/target", 10, 0, 0, 5),
            ],
        );
        assert_eq!(report.summary.roots, 2);
        assert_eq!(report.summary.total_bytes, 390);
        assert_eq!(report.summary.reclaimable_bytes, 160);
        assert_eq!(report.roots[0].reclaimable_bytes, 150);
    }

    #[test]
    fn empty_report_has_no_roots() {
        let report = build_report("/proj", Vec::new());
        assert!(report.roots.is_empty());
        assert_eq!(report.summary.reclaimable_bytes, 0);
    }
}
