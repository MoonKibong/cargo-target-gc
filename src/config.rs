//! `derust.toml` configuration loading.
//!
//! A missing config file resolves to [`Config::default`]; malformed TOML yields
//! a typed [`ConfigError`] instead of panicking.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Number of days an artifact must be untouched before it is considered stale.
const DEFAULT_RETENTION_DAYS: u64 = 14;

/// Effective derust configuration for target-artifact garbage collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Artifacts whose newest mtime is older than this many days are stale.
    pub retention_days: u64,
    /// Optional crate path to scope analysis to, relative to the project root.
    pub crate_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            retention_days: DEFAULT_RETENTION_DAYS,
            crate_path: None,
        }
    }
}

/// Errors that can occur while loading configuration.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// The config file exists but could not be read.
    Read { path: PathBuf, message: String },
    /// The config file exists but could not be parsed as TOML.
    Parse { path: PathBuf, message: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Read { path, message } => {
                write!(f, "failed to read {}: {message}", path.display())
            }
            ConfigError::Parse { path, message } => {
                write!(f, "failed to parse {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Load configuration for `root`, reading `<root>/derust.toml` when present.
pub fn load(root: &Path) -> Result<Config, ConfigError> {
    load_file(&root.join("derust.toml"))
}

/// Load configuration from an explicit file path; missing file → defaults.
pub fn load_file(path: &Path) -> Result<Config, ConfigError> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    parse(&text).map_err(|message| ConfigError::Parse {
        path: path.to_path_buf(),
        message,
    })
}

/// Parse config text, returning the parser's message on failure.
fn parse(text: &str) -> Result<Config, String> {
    toml::from_str(text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_yields_default() {
        let dir = std::env::temp_dir().join(format!("derust-cfg-missing-{}", std::process::id()));
        let config = load(&dir).expect("missing file is ok");
        assert_eq!(config, Config::default());
    }

    #[test]
    fn default_retention_is_fourteen_days() {
        assert_eq!(Config::default().retention_days, 14);
    }

    #[test]
    fn valid_file_parses_retention() {
        let config = parse("retention_days = 30\n").expect("parse");
        assert_eq!(config.retention_days, 30);
        // crate_path defaults to None when omitted.
        assert_eq!(config.crate_path, None);
    }

    #[test]
    fn crate_path_parses() {
        let config = parse("crate_path = \"crates/core\"\n").expect("parse");
        assert_eq!(config.crate_path, Some(PathBuf::from("crates/core")));
        // retention_days falls back to the default when omitted.
        assert_eq!(config.retention_days, 14);
    }

    #[test]
    fn invalid_toml_returns_err() {
        let result = parse("retention_days = [[[broken");
        assert!(result.is_err());
    }
}
