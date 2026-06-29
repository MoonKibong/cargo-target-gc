//! Read-only analysis of Cargo `target/` directories.
//!
//! This module NEVER invokes cargo and NEVER writes to disk. It walks a
//! validated `target/` root with `symlink_metadata` (no symlink following),
//! sums artifact sizes, and splits them into three categories so that `scan`
//! can report reclaimable space and `clean` can reuse the exact same walk to
//! decide what is safe to remove.

use std::fmt;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::discovery::Project;

/// How a single artifact group is classified by the read-only walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// `incremental/` subtrees — always reclaimable (cargo regenerates them).
    Incremental,
    /// Profile artifacts whose newest mtime is older than the retention window.
    Stale,
    /// Build-hot artifacts within the retention window — never reclaimable.
    Retained,
}

impl Category {
    /// Stable lowercase name used in reports and JSON.
    pub fn name(self) -> &'static str {
        match self {
            Category::Incremental => "incremental",
            Category::Stale => "stale",
            Category::Retained => "retained",
        }
    }

    /// Whether artifacts in this category may be removed by `clean`.
    pub fn is_reclaimable(self) -> bool {
        matches!(self, Category::Incremental | Category::Stale)
    }
}

/// A single deletable unit discovered within a target root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// Absolute-or-relative path to the artifact (file or directory).
    pub path: PathBuf,
    /// Classification of the artifact.
    pub category: Category,
    /// Total bytes the artifact occupies on disk (recursive, no symlink follow).
    pub bytes: u64,
}

/// The result of analyzing one target root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootAnalysis {
    /// The validated `target/` root that was analyzed.
    pub root: PathBuf,
    /// Total bytes across every category.
    pub total_bytes: u64,
    /// Bytes in `incremental/` subtrees.
    pub incremental_bytes: u64,
    /// Bytes in stale profile artifacts.
    pub stale_bytes: u64,
    /// Bytes in retained (build-hot) artifacts.
    pub retained_bytes: u64,
    /// Every categorized deletable unit, in walk order.
    pub artifacts: Vec<Artifact>,
}

impl RootAnalysis {
    /// Estimated reclaimable bytes (`Incremental` + `Stale`).
    pub fn reclaimable_bytes(&self) -> u64 {
        self.incremental_bytes + self.stale_bytes
    }

    /// Artifacts eligible for removal. Incremental is always included; stale is
    /// included only when `include_stale` is set.
    pub fn reclaimable(&self, include_stale: bool) -> impl Iterator<Item = &Artifact> {
        self.artifacts.iter().filter(move |a| match a.category {
            Category::Incremental => true,
            Category::Stale => include_stale,
            Category::Retained => false,
        })
    }
}

/// Errors raised while locating or analyzing target roots.
#[derive(Debug)]
pub enum TargetError {
    /// A directory could not be read during analysis.
    Read { path: PathBuf, source: io::Error },
    /// A configured `crate_path` escaped the discovered project root (absolute
    /// path or parent traversal). Scoping outside the project is never allowed.
    UnsafeCratePath { crate_path: PathBuf },
}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetError::Read { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
            TargetError::UnsafeCratePath { crate_path } => write!(
                f,
                "crate_path {} escapes the project root; it must be a relative \
                 path inside the discovered Cargo project/workspace",
                crate_path.display()
            ),
        }
    }
}

impl std::error::Error for TargetError {}

/// Validate that `path` is a real Cargo `target/` directory.
///
/// A directory qualifies when its basename is `target` and either an adjacent
/// `Cargo.toml` sits beside it or it contains cargo's `CACHEDIR.TAG` marker.
/// This is the single safety predicate `clean` relies on before deleting.
pub fn is_target_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    if path.file_name().and_then(|n| n.to_str()) != Some("target") {
        return false;
    }
    let adjacent_manifest = path
        .parent()
        .map(|p| p.join("Cargo.toml").is_file())
        .unwrap_or(false);
    adjacent_manifest || path.join("CACHEDIR.TAG").is_file()
}

/// Locate every validated, de-duplicated `target/` root for a project.
///
/// Considers the workspace-shared `<root>/target` plus any per-crate
/// `<crate>/target`. When `crate_path` is set, scoping is limited to that
/// crate's target. A `crate_path` that is absolute or escapes the project root
/// via parent traversal is rejected with [`TargetError::UnsafeCratePath`] so a
/// project's `derust.toml` can never point scan/clean outside its own tree.
/// Non-existent or invalid candidates are dropped.
pub fn locate_roots(
    project: &Project,
    crate_path: &Option<PathBuf>,
) -> Result<Vec<PathBuf>, TargetError> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    match crate_path {
        Some(rel) => {
            let crate_dir = contained_crate_dir(&project.root, rel).ok_or_else(|| {
                TargetError::UnsafeCratePath {
                    crate_path: rel.clone(),
                }
            })?;
            candidates.push(crate_dir.join("target"));
        }
        None => {
            candidates.push(project.root.join("target"));
            for krate in &project.crates {
                candidates.push(krate.dir.join("target"));
            }
        }
    }

    let mut roots: Vec<PathBuf> = Vec::new();
    let mut seen: Vec<PathBuf> = Vec::new();
    for candidate in candidates {
        if !is_target_dir(&candidate) {
            continue;
        }
        // De-duplicate by canonical path so a workspace-shared target counted
        // once via the root and once via a member is not analyzed twice.
        let key = std::fs::canonicalize(&candidate).unwrap_or_else(|_| candidate.clone());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        roots.push(candidate);
    }
    Ok(roots)
}

/// Resolve a configured `crate_path` against `root`, returning the crate
/// directory only when it stays lexically inside `root`.
///
/// Validation is purely lexical (no filesystem access) so it works even before
/// the crate directory exists: absolute paths, filesystem roots/prefixes, and
/// any `..` sequence that would climb above `root` are rejected. A `..` that is
/// re-covered by a later component (e.g. `crates/../core`) stays contained and
/// is allowed.
fn contained_crate_dir(root: &Path, rel: &Path) -> Option<PathBuf> {
    let mut depth: usize = 0;
    for component in rel.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(_) => depth += 1,
            Component::ParentDir => {
                // Climbing above the project root would escape the boundary.
                depth = depth.checked_sub(1)?;
            }
            // Absolute roots and Windows prefixes anchor outside `root`.
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(root.join(rel))
}

/// Walk a validated target `root` read-only and classify its artifacts.
///
/// Artifacts older than `retention_days` (by newest mtime within their profile)
/// are `Stale`; `incremental/` subtrees are always `Incremental`; everything
/// else is `Retained`. The walk uses `symlink_metadata` and never follows
/// symlinked directories.
pub fn analyze(root: &Path, retention_days: u64) -> Result<RootAnalysis, TargetError> {
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(retention_days.saturating_mul(86_400)))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut analysis = RootAnalysis {
        root: root.to_path_buf(),
        total_bytes: 0,
        incremental_bytes: 0,
        stale_bytes: 0,
        retained_bytes: 0,
        artifacts: Vec::new(),
    };

    for entry in read_dir(root)? {
        let path = entry.path();
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_file() {
            // Top-level bookkeeping files (CACHEDIR.TAG, .rustc_info.json) are
            // tiny and load-bearing; always retain them.
            let bytes = meta.len();
            analysis.retained_bytes += bytes;
            analysis.total_bytes += bytes;
            analysis.artifacts.push(Artifact {
                path,
                category: Category::Retained,
                bytes,
            });
            continue;
        }
        if meta.is_dir() {
            analyze_profile(&path, cutoff, &mut analysis)?;
        }
    }

    Ok(analysis)
}

/// Classify one top-level profile directory (e.g. `target/debug`).
///
/// `incremental/` is split out as its own always-reclaimable artifact; the rest
/// of the profile's children share a single staleness decision based on the
/// newest mtime among them, so a profile is reclaimed as a coherent unit.
fn analyze_profile(
    profile: &Path,
    cutoff: SystemTime,
    analysis: &mut RootAnalysis,
) -> Result<(), TargetError> {
    let mut rest: Vec<(PathBuf, u64)> = Vec::new();
    let mut rest_newest: Option<SystemTime> = None;

    for entry in read_dir(profile)? {
        let path = entry.path();
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        let (bytes, newest) = size_and_mtime(&path, &meta);
        if meta.is_dir() && path.file_name().and_then(|n| n.to_str()) == Some("incremental") {
            analysis.incremental_bytes += bytes;
            analysis.total_bytes += bytes;
            analysis.artifacts.push(Artifact {
                path,
                category: Category::Incremental,
                bytes,
            });
        } else {
            rest_newest = Some(match rest_newest {
                Some(cur) => cur.max(newest),
                None => newest,
            });
            rest.push((path, bytes));
        }
    }

    let category = match rest_newest {
        Some(newest) if newest < cutoff => Category::Stale,
        _ => Category::Retained,
    };
    for (path, bytes) in rest {
        match category {
            Category::Stale => analysis.stale_bytes += bytes,
            _ => analysis.retained_bytes += bytes,
        }
        analysis.total_bytes += bytes;
        analysis.artifacts.push(Artifact {
            path,
            category,
            bytes,
        });
    }
    Ok(())
}

/// Read a directory, mapping I/O failure to a typed [`TargetError`].
fn read_dir(path: &Path) -> Result<Vec<std::fs::DirEntry>, TargetError> {
    let mut entries = Vec::new();
    let iter = std::fs::read_dir(path).map_err(|source| TargetError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    for entry in iter {
        match entry {
            Ok(e) => entries.push(e),
            Err(_) => continue,
        }
    }
    Ok(entries)
}

/// Recursively total a path's bytes and find the newest mtime among the *files*
/// it contains (no symlink follow). Directory mtimes are ignored because a
/// freshly created parent directory would otherwise mask genuinely old build
/// artifacts. Unreadable entries are skipped best-effort, like `du`.
fn size_and_mtime(path: &Path, meta: &std::fs::Metadata) -> (u64, SystemTime) {
    if meta.is_file() {
        return (meta.len(), mtime_of(meta));
    }
    let mut total = 0u64;
    // Seed with the epoch so an empty directory reads as "old", and only file
    // mtimes can advance the newest timestamp.
    let mut newest = SystemTime::UNIX_EPOCH;
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return (total, newest),
    };
    for entry in entries.flatten() {
        let child = entry.path();
        let child_meta = match std::fs::symlink_metadata(&child) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if child_meta.file_type().is_symlink() {
            continue;
        }
        let (bytes, child_newest) = size_and_mtime(&child, &child_meta);
        total += bytes;
        if child_newest > newest {
            newest = child_newest;
        }
    }
    (total, newest)
}

/// Modification time of `meta`, defaulting to the epoch when unavailable.
fn mtime_of(meta: &std::fs::Metadata) -> SystemTime {
    meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::Duration;

    /// Build a unique temp directory for a test.
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "derust-target-{tag}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Write `bytes` of content to `path`, creating parents, and set its mtime.
    fn write_aged(path: &Path, len: usize, age_days: u64) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parents");
        }
        fs::write(path, vec![b'x'; len]).expect("write file");
        let when = SystemTime::now()
            .checked_sub(Duration::from_secs(age_days * 86_400))
            .expect("aged time");
        File::options()
            .write(true)
            .open(path)
            .expect("open for mtime")
            .set_modified(when)
            .expect("set mtime");
    }

    /// Create a fixture project with a populated `target/` tree.
    /// Returns the project root; the caller removes it.
    fn fixture_project(tag: &str) -> PathBuf {
        let root = temp_dir(tag);
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"x\"\n").expect("manifest");
        let target = root.join("target");
        fs::create_dir_all(&target).expect("target");
        fs::write(target.join("CACHEDIR.TAG"), "Signature").expect("tag");
        // Fresh (retained) profile.
        write_aged(&target.join("debug/deps/lib.rlib"), 1000, 0);
        write_aged(&target.join("debug/incremental/seg/x.o"), 500, 0);
        // Stale profile (very old).
        write_aged(&target.join("release/deps/lib.rlib"), 2000, 100);
        write_aged(&target.join("release/incremental/seg/y.o"), 700, 100);
        root
    }

    #[test]
    fn is_target_dir_requires_target_basename() {
        let root = temp_dir("isdir");
        fs::write(root.join("Cargo.toml"), "x").expect("manifest");
        let target = root.join("target");
        fs::create_dir_all(&target).expect("target");
        assert!(is_target_dir(&target));
        assert!(!is_target_dir(&root));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn analyze_splits_categories_by_age() {
        let root = fixture_project("splits");
        let target = root.join("target");
        let a = analyze(&target, 14).expect("analyze");

        // Incremental = both incremental subtrees regardless of age.
        assert_eq!(a.incremental_bytes, 500 + 700);
        // debug/deps is fresh → retained; release/deps is old → stale.
        assert_eq!(a.retained_bytes, 1000 + "Signature".len() as u64);
        assert_eq!(a.stale_bytes, 2000);
        assert_eq!(a.reclaimable_bytes(), 500 + 700 + 2000);
        assert_eq!(
            a.total_bytes,
            a.incremental_bytes + a.stale_bytes + a.retained_bytes
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn contained_crate_dir_allows_relative_inside_root() {
        let root = Path::new("/proj");
        assert_eq!(
            contained_crate_dir(root, Path::new("crates/core")),
            Some(PathBuf::from("/proj/crates/core"))
        );
        // A `..` re-covered by a later component stays inside the root.
        assert_eq!(
            contained_crate_dir(root, Path::new("crates/../core")),
            Some(PathBuf::from("/proj/crates/../core"))
        );
        // A bare `.` is contained and resolves to the root itself.
        assert_eq!(
            contained_crate_dir(root, Path::new(".")),
            Some(PathBuf::from("/proj"))
        );
    }

    #[test]
    fn contained_crate_dir_rejects_escapes() {
        let root = Path::new("/proj");
        // Parent traversal that climbs above the root.
        assert_eq!(contained_crate_dir(root, Path::new("../evil")), None);
        assert_eq!(
            contained_crate_dir(root, Path::new("crates/../../evil")),
            None
        );
        // Absolute paths anchor outside the root.
        assert_eq!(contained_crate_dir(root, Path::new("/etc")), None);
    }

    #[test]
    fn locate_roots_rejects_escaping_crate_path() {
        let root = fixture_project("escape");
        let project = Project {
            root: root.clone(),
            manifest: root.join("Cargo.toml"),
            kind: crate::discovery::ProjectKind::Package,
            crates: vec![crate::discovery::CrateManifest {
                dir: root.clone(),
                manifest: root.join("Cargo.toml"),
            }],
        };
        let result = locate_roots(&project, &Some(PathBuf::from("../evil")));
        assert!(matches!(result, Err(TargetError::UnsafeCratePath { .. })));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn locate_roots_accepts_contained_crate_path() {
        let root = fixture_project("scoped");
        // Build a nested crate with its own target so a contained crate_path resolves.
        let crate_dir = root.join("crates/core");
        let crate_target = crate_dir.join("target");
        fs::create_dir_all(&crate_target).expect("crate target");
        fs::write(crate_dir.join("Cargo.toml"), "[package]\nname=\"core\"\n").expect("manifest");
        fs::write(crate_target.join("CACHEDIR.TAG"), "Signature").expect("tag");

        let project = Project {
            root: root.clone(),
            manifest: root.join("Cargo.toml"),
            kind: crate::discovery::ProjectKind::Package,
            crates: vec![crate::discovery::CrateManifest {
                dir: root.clone(),
                manifest: root.join("Cargo.toml"),
            }],
        };
        let roots =
            locate_roots(&project, &Some(PathBuf::from("crates/core"))).expect("scoped roots");
        assert_eq!(roots, vec![crate_target]);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn reclaimable_excludes_stale_unless_opted_in() {
        let root = fixture_project("recl");
        let target = root.join("target");
        let a = analyze(&target, 14).expect("analyze");

        let incremental_only: u64 = a.reclaimable(false).map(|x| x.bytes).sum();
        assert_eq!(incremental_only, 500 + 700);
        let with_stale: u64 = a.reclaimable(true).map(|x| x.bytes).sum();
        assert_eq!(with_stale, 500 + 700 + 2000);
        // No reclaimable artifact is ever Retained.
        assert!(a
            .reclaimable(true)
            .all(|x| x.category != Category::Retained));
        let _ = fs::remove_dir_all(&root);
    }
}
