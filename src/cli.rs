//! Command-line argument definitions for the `cargo-target-gc` binary.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// cargo-target-gc — Cargo target-artifact garbage collection without throwing away hot build cache.
#[derive(Debug, Parser)]
#[command(
    name = "cargo-target-gc",
    version,
    about,
    long_about = None,
    after_help = "Run `cargo target-gc` from the same directory where you would run `cargo build`.\nFor projects built through wrappers such as `make`, cd into the Cargo project\nor workspace directory used by that wrapper and run it there."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Analyze target directories and estimate space that can be reclaimed without deleting hot artifacts.
    #[command(
        after_help = "Run this from the Cargo project/workspace root: the same directory where `cargo build` would create target/. cargo-target-gc does not search opaque wrapper build paths."
    )]
    Scan {
        /// Cargo project or workspace path to inspect (defaults to the current directory).
        #[arg(long)]
        path: Option<PathBuf>,
        /// Emit raw-byte JSON instead of the human-readable storage report.
        #[arg(long)]
        json: bool,
    },
    /// Remove reclaimable target artifacts after an explicit dry-run or confirmation.
    #[command(
        after_help = "Run this from the Cargo project/workspace root: the same directory where `cargo build` would create target/. Use --dry-run first when checking an unfamiliar project."
    )]
    Clean {
        /// Cargo project or workspace path to clean (defaults to the current directory).
        #[arg(long)]
        path: Option<PathBuf>,
        /// Emit raw-byte JSON instead of the human-readable cleanup report.
        #[arg(long)]
        json: bool,
        /// Preview removals without deleting anything; cannot be combined with --confirm.
        #[arg(long, conflicts_with = "confirm")]
        dry_run: bool,
        /// Execute removals for reclaimable artifacts inside validated target roots.
        #[arg(long)]
        confirm: bool,
        /// Allow cleaning even when an active Cargo/rustc process is detected.
        #[arg(long)]
        force_active: bool,
        /// Refuse confirmed cleanup above this size unless increased (for example: 500MiB, 10GiB).
        #[arg(long, value_parser = parse_size)]
        max_reclaim: Option<u64>,
        /// Also reclaim stale profile artifacts, not just incremental caches.
        #[arg(long)]
        stale: bool,
        /// Also reclaim fresh incremental cache and Cargo profile cache directories.
        #[arg(long)]
        profile_cache: bool,
    },
    /// Show the effective retention and crate-scope configuration.
    Config {
        /// Cargo project path whose target-gc.toml should be resolved (defaults to cwd).
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Install host-agent skills that teach agents how to safely run cargo-target-gc.
    InstallAgentSkills {
        /// Claude Code skills directory (defaults to ~/.claude/skills).
        #[arg(long)]
        claude_skills_dir: Option<PathBuf>,
        /// Codex skills directory (defaults to ~/.codex/skills).
        #[arg(long)]
        codex_skills_dir: Option<PathBuf>,
        /// Install for all supported hosts instead of only detected hosts.
        #[arg(long, conflicts_with = "only")]
        all: bool,
        /// Install only for selected hosts, comma-separated: claude,codex.
        #[arg(long, value_name = "HOSTS")]
        only: Option<String>,
        /// Approve all detected host installs without prompting.
        #[arg(long)]
        yes: bool,
        /// Print planned installs without writing files.
        #[arg(long)]
        dry_run: bool,
        /// Overwrite existing cargo-target-gc skills without prompting.
        #[arg(long, conflicts_with = "skip_existing")]
        force: bool,
        /// Keep existing cargo-target-gc skills without prompting.
        #[arg(long)]
        skip_existing: bool,
    },
}

/// Parse a byte size with optional binary suffix: B, KiB, MiB, GiB, TiB.
fn parse_size(value: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("size cannot be empty".into());
    }
    let split_at = trimmed
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let (digits, suffix) = trimmed.split_at(split_at);
    if digits.is_empty() {
        return Err(format!("size {value:?} must start with a number"));
    }
    let base: u64 = digits
        .parse()
        .map_err(|_| format!("size {value:?} is not a valid integer"))?;
    let multiplier = match suffix.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "kib" | "k" | "kb" => 1024,
        "mib" | "m" | "mb" => 1024_u64.pow(2),
        "gib" | "g" | "gb" => 1024_u64.pow(3),
        "tib" | "t" | "tb" => 1024_u64.pow(4),
        other => return Err(format!("unsupported size suffix {other:?}")),
    };
    base.checked_mul(multiplier)
        .ok_or_else(|| format!("size {value:?} is too large"))
}
