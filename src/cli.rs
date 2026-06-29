//! Command-line argument definitions for the `derust` binary.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// derust — a read-only Cargo target-artifact garbage collector.
#[derive(Debug, Parser)]
#[command(name = "derust", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Analyze `target/` directories and report reclaimable space (read-only).
    Scan {
        /// Project path to scan (defaults to the current directory).
        #[arg(long)]
        path: Option<PathBuf>,
        /// Emit the report as JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
    /// Remove reclaimable target artifacts (requires --dry-run or --confirm).
    Clean {
        /// Project path to clean (defaults to the current directory).
        #[arg(long)]
        path: Option<PathBuf>,
        /// Emit the summary as JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
        /// Preview removals without deleting anything (cannot be combined
        /// with --confirm).
        #[arg(long, conflicts_with = "confirm")]
        dry_run: bool,
        /// Execute removals (deletes reclaimable artifacts).
        #[arg(long)]
        confirm: bool,
        /// Also reclaim stale profile artifacts, not just incremental ones.
        #[arg(long)]
        stale: bool,
    },
    /// Show the effective derust configuration.
    Config {
        /// Project path whose derust.toml should be resolved (defaults to cwd).
        #[arg(long)]
        path: Option<PathBuf>,
    },
}
