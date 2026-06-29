//! `clean` — preview and (with confirmation) remove reclaimable target
//! artifacts.
//!
//! Removal REUSES [`crate::target::analyze`] so the deletion set is exactly the
//! reclaimable artifacts the scan reports — there is no second walk and no
//! second categorization. Every planned removal is guarded to lie strictly
//! inside a validated cargo `target/` root before any deletion occurs.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::config::{self, ConfigError};
use crate::discovery::{self, DiscoveryError};
use crate::target::{self, Category, TargetError};

/// One artifact slated for removal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removal {
    /// Path to remove (always inside a validated target root).
    pub path: PathBuf,
    /// Why the artifact is reclaimable.
    pub category: Category,
    /// Bytes the artifact occupies.
    pub bytes: u64,
}

/// A computed, not-yet-executed clean plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanPlan {
    /// Project root the plan was computed for.
    pub project_root: PathBuf,
    /// Validated target roots considered.
    pub roots: Vec<PathBuf>,
    /// Artifacts that would be removed.
    pub removals: Vec<Removal>,
}

impl CleanPlan {
    /// Total bytes the plan would reclaim.
    pub fn total_bytes(&self) -> u64 {
        self.removals.iter().map(|r| r.bytes).sum()
    }
}

/// Errors raised while planning or executing a clean.
#[derive(Debug)]
pub enum CleanError {
    /// Project discovery failed.
    Discovery(DiscoveryError),
    /// Configuration loading failed.
    Config(ConfigError),
    /// A target root could not be analyzed.
    Target(TargetError),
    /// A planned path was not safely inside a validated target root.
    Unsafe { path: PathBuf },
    /// A removal failed during execution.
    Remove { path: PathBuf, source: io::Error },
}

impl fmt::Display for CleanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CleanError::Discovery(e) => write!(f, "{e}"),
            CleanError::Config(e) => write!(f, "{e}"),
            CleanError::Target(e) => write!(f, "{e}"),
            CleanError::Unsafe { path } => write!(
                f,
                "refusing to remove {}: not inside a validated cargo target/ root",
                path.display()
            ),
            CleanError::Remove { path, source } => {
                write!(f, "failed to remove {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for CleanError {}

impl From<DiscoveryError> for CleanError {
    fn from(e: DiscoveryError) -> Self {
        CleanError::Discovery(e)
    }
}

impl From<ConfigError> for CleanError {
    fn from(e: ConfigError) -> Self {
        CleanError::Config(e)
    }
}

impl From<TargetError> for CleanError {
    fn from(e: TargetError) -> Self {
        CleanError::Target(e)
    }
}

/// Compute a clean plan for the project rooted at or above `path`.
///
/// Incremental artifacts are always included; stale artifacts only when
/// `include_stale` is set. Every removal is validated to live inside one of the
/// discovered target roots.
pub fn plan(path: &Path, include_stale: bool) -> Result<CleanPlan, CleanError> {
    let project = discovery::discover(path)?;
    let cfg = config::load(&project.root)?;
    let roots = target::locate_roots(&project, &cfg.crate_path)?;

    let mut removals = Vec::new();
    for root in &roots {
        let analysis = target::analyze(root, cfg.retention_days)?;
        for artifact in analysis.reclaimable(include_stale) {
            guard_inside_root(&artifact.path, root)?;
            removals.push(Removal {
                path: artifact.path.clone(),
                category: artifact.category,
                bytes: artifact.bytes,
            });
        }
    }

    Ok(CleanPlan {
        project_root: project.root,
        roots,
        removals,
    })
}

/// Hard safety guard: `path` must be a strict descendant of `root`, and `root`
/// must be a validated cargo `target/` directory. Never deletes a root itself.
fn guard_inside_root(path: &Path, root: &Path) -> Result<(), CleanError> {
    let unsafe_err = || CleanError::Unsafe {
        path: path.to_path_buf(),
    };
    if !target::is_target_dir(root) {
        return Err(unsafe_err());
    }
    let root_canon = std::fs::canonicalize(root).map_err(|_| unsafe_err())?;
    let path_canon = std::fs::canonicalize(path).map_err(|_| unsafe_err())?;
    if path_canon == root_canon || !path_canon.starts_with(&root_canon) {
        return Err(unsafe_err());
    }
    Ok(())
}

/// Execute a clean plan, removing each artifact and returning the bytes freed.
///
/// Each path is re-guarded immediately before removal so a stale plan cannot
/// delete outside a validated target root.
pub fn execute(plan: &CleanPlan) -> Result<u64, CleanError> {
    let mut freed = 0u64;
    for removal in &plan.removals {
        let root = plan
            .roots
            .iter()
            .find(|r| removal.path.starts_with(r))
            .ok_or_else(|| CleanError::Unsafe {
                path: removal.path.clone(),
            })?;
        guard_inside_root(&removal.path, root)?;
        remove(&removal.path)?;
        freed += removal.bytes;
    }
    Ok(freed)
}

/// Remove a file or directory tree.
fn remove(path: &Path) -> Result<(), CleanError> {
    let meta = std::fs::symlink_metadata(path).map_err(|source| CleanError::Remove {
        path: path.to_path_buf(),
        source,
    })?;
    let result = if meta.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    result.map_err(|source| CleanError::Remove {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{Duration, SystemTime};

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("derust-clean-{tag}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn write_aged(path: &Path, len: usize, age_days: u64) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parents");
        }
        fs::write(path, vec![b'x'; len]).expect("write");
        let when = SystemTime::now()
            .checked_sub(Duration::from_secs(age_days * 86_400))
            .expect("aged");
        File::options()
            .write(true)
            .open(path)
            .expect("open")
            .set_modified(when)
            .expect("mtime");
    }

    fn project(tag: &str) -> PathBuf {
        let root = temp_dir(tag);
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"x\"\n").expect("manifest");
        let target = root.join("target");
        fs::create_dir_all(&target).expect("target");
        fs::write(target.join("CACHEDIR.TAG"), "Signature").expect("tag");
        write_aged(&target.join("debug/deps/lib.rlib"), 1000, 0); // retained
        write_aged(&target.join("debug/incremental/seg/x.o"), 500, 0); // incremental
        write_aged(&target.join("release/deps/lib.rlib"), 2000, 100); // stale
        root
    }

    #[test]
    fn plan_default_targets_incremental_only() {
        let root = project("incr");
        let plan = plan(&root, false).expect("plan");
        assert_eq!(plan.total_bytes(), 500);
        assert!(plan
            .removals
            .iter()
            .all(|r| r.category == Category::Incremental));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn plan_with_stale_includes_stale() {
        let root = project("stale");
        let plan = plan(&root, true).expect("plan");
        assert_eq!(plan.total_bytes(), 500 + 2000);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_removes_reclaimable_preserves_retained() {
        let root = project("exec");
        let target = root.join("target");
        let plan = plan(&root, true).expect("plan");
        let freed = execute(&plan).expect("execute");
        assert_eq!(freed, 500 + 2000);
        // Retained debug/deps survives; reclaimable artifacts are gone.
        assert!(target.join("debug/deps/lib.rlib").exists());
        assert!(!target.join("debug/incremental").exists());
        assert!(!target.join("release/deps/lib.rlib").exists());
        // The target root and its tag are never removed.
        assert!(target.join("CACHEDIR.TAG").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn guard_rejects_path_outside_target() {
        let root = project("guard");
        let target = root.join("target");
        // A sibling path outside the target root must be rejected.
        let outside = root.join("Cargo.toml");
        assert!(matches!(
            guard_inside_root(&outside, &target),
            Err(CleanError::Unsafe { .. })
        ));
        // The root itself must never be deletable.
        assert!(matches!(
            guard_inside_root(&target, &target),
            Err(CleanError::Unsafe { .. })
        ));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn guard_rejects_non_target_root() {
        let root = project("nontarget");
        // `root` is a project dir, not a `target/` dir → not a valid root.
        let inside = root.join("Cargo.toml");
        assert!(matches!(
            guard_inside_root(&inside, &root),
            Err(CleanError::Unsafe { .. })
        ));
        let _ = fs::remove_dir_all(&root);
    }
}
