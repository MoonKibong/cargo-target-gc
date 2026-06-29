//! `derust.toml` configuration loading.
//!
//! A missing config file resolves to [`Config::default`]; malformed TOML yields
//! a typed [`ConfigError`] instead of panicking.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Per-check enable toggles. Every check is enabled by default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Checks {
    /// Run `cargo check`.
    pub check: bool,
    /// Run `cargo test`.
    pub test: bool,
    /// Run `cargo fmt --check`.
    pub fmt: bool,
    /// Run `cargo clippy -- -D warnings`.
    pub clippy: bool,
}

impl Default for Checks {
    fn default() -> Self {
        Checks {
            check: true,
            test: true,
            fmt: true,
            clippy: true,
        }
    }
}

/// Effective derust configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Which checks the scan should run.
    pub checks: Checks,
    /// Optional crate path to target relative to the project root.
    pub crate_path: Option<PathBuf>,
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
    fn valid_file_parses_toggles() {
        let config = parse("[checks]\nclippy = false\ntest = false\n").expect("parse");
        assert!(config.checks.check);
        assert!(!config.checks.clippy);
        assert!(!config.checks.test);
    }

    #[test]
    fn crate_path_parses() {
        let config = parse("crate_path = \"crates/core\"\n").expect("parse");
        assert_eq!(config.crate_path, Some(PathBuf::from("crates/core")));
    }

    #[test]
    fn invalid_toml_returns_err() {
        let result = parse("checks = [[[broken");
        assert!(result.is_err());
    }
}
