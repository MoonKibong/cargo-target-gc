//! Command-line argument definitions for the `derust` binary.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// derust — a read-only Rust project health & refactoring-readiness CLI.
#[derive(Debug, Parser)]
#[command(name = "derust", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run read-only health checks over a Rust project or workspace.
    Scan {
        /// Project path to scan (defaults to the current directory).
        #[arg(long)]
        path: Option<PathBuf>,
        /// Emit the report as JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
    /// Show the effective derust configuration.
    Config {
        /// Project path whose derust.toml should be resolved (defaults to cwd).
        #[arg(long)]
        path: Option<PathBuf>,
    },
}
