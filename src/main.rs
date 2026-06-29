//! `cargo-target-gc` binary entry point — a read-only Cargo target-artifact GC CLI.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;

use cargo_target_gc::agent_skill::{self, InstallOptions, Selection, SkillCollision};
use cargo_target_gc::clean::{self, CleanPlan};
use cargo_target_gc::cli::{Cli, Command};
use cargo_target_gc::report::human;
use cargo_target_gc::{config, discovery, scan};

fn main() -> Result<()> {
    let cli = Cli::parse_from(cargo_args());
    match cli.command {
        Command::Scan { path, json } => run_scan(path, json),
        Command::Clean {
            path,
            json,
            dry_run,
            confirm,
            force_active,
            max_reclaim,
            stale,
        } => run_clean(
            path,
            json,
            dry_run,
            confirm,
            force_active,
            max_reclaim,
            stale,
        ),
        Command::Config { path } => run_config(path),
        Command::InstallAgentSkills {
            claude_skills_dir,
            codex_skills_dir,
            all,
            only,
            yes,
            dry_run,
            force,
            skip_existing,
        } => {
            let hosts = match only {
                Some(raw) => agent_skill::parse_hosts(&raw)?,
                None => Vec::new(),
            };
            let collision = if force {
                SkillCollision::Overwrite
            } else if skip_existing {
                SkillCollision::Skip
            } else {
                SkillCollision::Prompt
            };
            let selection = if all {
                Selection::All
            } else {
                Selection::Detected
            };
            let opts = InstallOptions {
                claude_skills_dir,
                codex_skills_dir,
                only: hosts,
                selection,
                collision,
                dry_run,
                yes,
            };
            agent_skill::install(&opts)
        }
    }
}

fn cargo_args() -> Vec<OsString> {
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.get(1).and_then(|arg| arg.to_str()) == Some("target-gc") {
        args.remove(1);
    }
    args
}

/// Resolve an optional path argument to the current directory when absent.
fn resolve(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| PathBuf::from("."))
}

/// Handle `cargo target-gc scan`. Report → stdout; progress → stderr (inside `scan`).
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

/// Handle `cargo target-gc clean`.
///
/// Refuses (nonzero exit) unless exactly one of `--dry-run` / `--confirm` is
/// given; clap rejects passing both, and this function rejects passing neither.
/// `--dry-run` previews and deletes nothing; `--confirm` executes.
fn run_clean(
    path: Option<PathBuf>,
    json: bool,
    dry_run: bool,
    confirm: bool,
    force_active: bool,
    max_reclaim: Option<u64>,
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
        eprintln!(
            "cargo-target-gc: removing {} artifact(s)",
            plan.removals.len()
        );
        let freed =
            clean::execute(&plan, force_active, max_reclaim).context("executing clean plan")?;
        emit_clean(&plan, true, freed, json)?;
    } else {
        eprintln!("cargo-target-gc: dry-run — no files will be removed");
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
    println!("cargo target-gc clean: {}", plan.project_root.display());
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

/// Handle `cargo target-gc config`.
///
/// Discovers the Cargo project root first (like `scan`) so the effective config
/// reflects the project's `target-gc.toml` even when run from a subdirectory.
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
    println!("cargo target-gc config: {}", root.display());
    println!("  retention_days: {}", cfg.retention_days);
    println!(
        "  incremental_retention_hours: {}",
        cfg.incremental_retention_hours
    );
    match cfg.max_reclaim_bytes {
        Some(bytes) => println!("  max_reclaim_bytes: {}", bytes),
        None => println!("  max_reclaim_bytes: (none)"),
    }
    match &cfg.crate_path {
        Some(p) => println!("  crate_path:     {}", p.display()),
        None => println!("  crate_path:     (none)"),
    }
}
