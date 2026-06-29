//! `derust` binary entry point — a read-only Rust project health scan CLI.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use derust::cli::{Cli, Command};
use derust::{config, discovery, scan};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan { path, json } => run_scan(path, json),
        Command::Config { path } => run_config(path),
    }
}

/// Resolve an optional path argument to the current directory when absent.
fn resolve(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| PathBuf::from("."))
}

/// Handle `derust scan`.
fn run_scan(path: Option<PathBuf>, json: bool) -> Result<()> {
    let root = resolve(path);
    let report =
        scan::scan(&root).with_context(|| format!("scan failed for {}", root.display()))?;
    if json {
        println!("{}", report.render_json().context("render JSON report")?);
    } else {
        print!("{}", report.render_text());
    }
    Ok(())
}

/// Handle `derust config`.
///
/// Discovers the Cargo project root first (like `scan`) so the effective config
/// reflects the project's `derust.toml` even when run from a subdirectory.
fn run_config(path: Option<PathBuf>) -> Result<()> {
    let start = resolve(path);
    let project = discovery::discover(&start)
        .with_context(|| format!("discover project for {}", start.display()))?;
    let cfg = config::load(&project.root)
        .with_context(|| format!("load config for {}", project.root.display()))?;
    print_config(&project.root, &cfg);
    Ok(())
}

/// Print the effective configuration in a stable, readable form.
fn print_config(root: &Path, cfg: &config::Config) {
    println!("derust config: {}", root.display());
    println!("  checks.check:  {}", cfg.checks.check);
    println!("  checks.test:   {}", cfg.checks.test);
    println!("  checks.fmt:    {}", cfg.checks.fmt);
    println!("  checks.clippy: {}", cfg.checks.clippy);
    match &cfg.crate_path {
        Some(p) => println!("  crate_path:    {}", p.display()),
        None => println!("  crate_path:    (none)"),
    }
}
