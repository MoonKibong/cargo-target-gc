//! `derust` binary entry point — a read-only Cargo target-artifact GC CLI.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;

use derust::clean::{self, CleanPlan};
use derust::cli::{Cli, Command};
use derust::report::human;
use derust::{config, discovery, scan};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan { path, json } => run_scan(path, json),
        Command::Clean {
            path,
            json,
            dry_run,
            confirm,
            stale,
        } => run_clean(path, json, dry_run, confirm, stale),
        Command::Config { path } => run_config(path),
    }
}

/// Resolve an optional path argument to the current directory when absent.
fn resolve(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| PathBuf::from("."))
}

/// Handle `derust scan`. Report → stdout; progress → stderr (inside `scan`).
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

/// Handle `derust clean`.
///
/// Refuses (nonzero exit) unless exactly one of `--dry-run` / `--confirm` is
/// given; clap rejects passing both, and this function rejects passing neither.
/// `--dry-run` previews and deletes nothing; `--confirm` executes.
fn run_clean(
    path: Option<PathBuf>,
    json: bool,
    dry_run: bool,
    confirm: bool,
    stale: bool,
) -> Result<()> {
    if !dry_run && !confirm {
        bail!(
            "clean requires an explicit mode: pass --dry-run to preview removals \
             or --confirm to delete reclaimable artifacts"
        );
    }

    let root = resolve(path);
    let plan = clean::plan(&root, stale)
        .with_context(|| format!("clean planning failed for {}", root.display()))?;

    // clap guarantees --dry-run and --confirm are mutually exclusive, so
    // `confirm` here unambiguously selects execution over previewing.
    if confirm {
        eprintln!("derust: removing {} artifact(s)", plan.removals.len());
        let freed = clean::execute(&plan).context("executing clean plan")?;
        emit_clean(&plan, true, freed, json)?;
    } else {
        eprintln!("derust: dry-run — no files will be removed");
        emit_clean(&plan, false, plan.total_bytes(), json)?;
    }
    Ok(())
}

/// Render a clean plan/result to stdout as text or JSON.
fn emit_clean(plan: &CleanPlan, executed: bool, bytes: u64, json: bool) -> Result<()> {
    if json {
        let removals: Vec<_> = plan
            .removals
            .iter()
            .map(|r| {
                serde_json::json!({
                    "path": r.path.display().to_string(),
                    "category": r.category.name(),
                    "bytes": r.bytes,
                })
            })
            .collect();
        let value = serde_json::json!({
            "project_root": plan.project_root.display().to_string(),
            "executed": executed,
            "reclaimable_bytes": bytes,
            "removals": removals,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&value).context("render clean JSON")?
        );
        return Ok(());
    }

    let verb = if executed { "removed" } else { "would remove" };
    println!("derust clean: {}", plan.project_root.display());
    for removal in &plan.removals {
        println!(
            "  {verb} {:<12} {}  ({})",
            removal.category.name(),
            removal.path.display(),
            human(removal.bytes)
        );
    }
    let label = if executed { "reclaimed" } else { "reclaimable" };
    println!(
        "  {} artifact(s), {}: {}",
        plan.removals.len(),
        label,
        human(bytes)
    );
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
    println!("  retention_days: {}", cfg.retention_days);
    match &cfg.crate_path {
        Some(p) => println!("  crate_path:     {}", p.display()),
        None => println!("  crate_path:     (none)"),
    }
}
