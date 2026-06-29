//! Rust project / workspace discovery by walking up to the nearest `Cargo.toml`.
//!
//! Read-only: this module only reads manifests, it never writes to disk.

use std::fmt;
use std::path::{Path, PathBuf};

/// The shape of a discovered Cargo project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// A single `[package]` crate.
    Package,
    /// A `[workspace]` aggregating member crates.
    Workspace,
}

/// A single crate manifest discovered within a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateManifest {
    /// Directory containing the crate's `Cargo.toml`.
    pub dir: PathBuf,
    /// Path to the crate's `Cargo.toml`.
    pub manifest: PathBuf,
}

/// A discovered Cargo project rooted at the nearest manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// Directory containing the root `Cargo.toml`.
    pub root: PathBuf,
    /// Path to the root `Cargo.toml`.
    pub manifest: PathBuf,
    /// Whether the root manifest is a package or a workspace.
    pub kind: ProjectKind,
    /// All crate manifests belonging to the project (>= 1).
    pub crates: Vec<CrateManifest>,
}

/// Errors that can occur while discovering a project.
#[derive(Debug, PartialEq, Eq)]
pub enum DiscoveryError {
    /// No `Cargo.toml` was found walking up from the start path.
    NotFound { start: PathBuf },
    /// A manifest could not be read from disk.
    Read { path: PathBuf, message: String },
    /// A manifest could not be parsed as TOML.
    Parse { path: PathBuf, message: String },
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryError::NotFound { start } => {
                write!(f, "no Cargo.toml found at or above {}", start.display())
            }
            DiscoveryError::Read { path, message } => {
                write!(f, "failed to read {}: {message}", path.display())
            }
            DiscoveryError::Parse { path, message } => {
                write!(f, "failed to parse {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for DiscoveryError {}

/// Walk up from `start` (a directory or file path) to the nearest `Cargo.toml`
/// and describe the project it roots.
pub fn discover(start: &Path) -> Result<Project, DiscoveryError> {
    let manifest = find_manifest(start).ok_or_else(|| DiscoveryError::NotFound {
        start: start.to_path_buf(),
    })?;
    let root = manifest
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let value = parse_manifest(&manifest)?;

    if let Some(workspace) = value.get("workspace") {
        let crates = workspace_members(&root, workspace);
        return Ok(Project {
            root,
            manifest,
            kind: ProjectKind::Workspace,
            crates,
        });
    }

    let crates = vec![CrateManifest {
        dir: root.clone(),
        manifest: manifest.clone(),
    }];
    Ok(Project {
        root,
        manifest,
        kind: ProjectKind::Package,
        crates,
    })
}

/// Locate the nearest `Cargo.toml` at or above `start`.
fn find_manifest(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent().map(Path::to_path_buf)
    } else {
        Some(start.to_path_buf())
    };
    while let Some(current) = dir {
        let candidate = current.join("Cargo.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = current.parent().map(Path::to_path_buf);
    }
    None
}

/// Read and parse a manifest into a TOML table.
fn parse_manifest(path: &Path) -> Result<toml::Value, DiscoveryError> {
    let text = std::fs::read_to_string(path).map_err(|e| DiscoveryError::Read {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    text.parse::<toml::Value>()
        .map_err(|e| DiscoveryError::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
}

/// Resolve workspace member directories into crate manifests.
fn workspace_members(root: &Path, workspace: &toml::Value) -> Vec<CrateManifest> {
    let members = workspace
        .get("members")
        .and_then(toml::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    members
        .iter()
        .filter_map(toml::Value::as_str)
        .map(|member| {
            let dir = root.join(member);
            let manifest = dir.join("Cargo.toml");
            CrateManifest { dir, manifest }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    #[test]
    fn single_package_finds_one_crate() {
        let project = discover(&fixtures().join("single-package")).expect("discover");
        assert_eq!(project.kind, ProjectKind::Package);
        assert_eq!(project.crates.len(), 1);
    }

    #[test]
    fn workspace_finds_two_members() {
        let project = discover(&fixtures().join("workspace")).expect("discover");
        assert_eq!(project.kind, ProjectKind::Workspace);
        assert_eq!(project.crates.len(), 2);
    }

    #[test]
    fn missing_manifest_returns_err() {
        // Use an isolated temp dir so we don't walk up into cargo-target-gc's own manifest.
        let dir = std::env::temp_dir().join(format!(
            "cargo-target-gc-no-manifest-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let result = discover(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(matches!(result, Err(DiscoveryError::NotFound { .. })));
    }

    #[test]
    fn walks_up_from_nested_file() {
        let nested = fixtures().join("single-package/src/main.rs");
        let project = discover(&nested).expect("discover from nested file");
        assert_eq!(project.kind, ProjectKind::Package);
    }
}
