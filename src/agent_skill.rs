//! Host-agent skill installer for cargo-target-gc.

use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCollision {
    Prompt,
    Overwrite,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Detected,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostId {
    Claude,
    Codex,
}

impl HostId {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "claude" | "claude-code" | "cc" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }
}

impl fmt::Display for HostId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostId::Claude => write!(f, "claude"),
            HostId::Codex => write!(f, "codex"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub claude_skills_dir: Option<PathBuf>,
    pub codex_skills_dir: Option<PathBuf>,
    pub only: Vec<HostId>,
    pub selection: Selection,
    pub collision: SkillCollision,
    pub dry_run: bool,
    pub yes: bool,
}

#[derive(Debug, Clone)]
pub struct PlannedInstall {
    pub host: HostId,
    pub label: &'static str,
    pub detected: bool,
    pub skill_file: PathBuf,
}

const SKILL_DIR_NAME: &str = "cargo-target-gc";

pub const AGENT_SKILL: &str = r#"---
name: cargo-target-gc
description: Use when Rust/Cargo builds, coding agents, Claude Code, Codex, Gemini CLI, or repeated cargo commands are growing disk usage in target/ directories and the user wants safe scan, dry-run cleanup, or confirmed Cargo build-artifact garbage collection.
---

# cargo-target-gc

Use this skill when the user asks to reduce Rust build disk usage, clean Cargo `target/`, inspect
large Rust build artifacts, or recover disk space after agentic coding sessions.

## Core Rules

- Start with `cargo target-gc scan`; it is read-only, never invokes Cargo, and creates no build artifacts.
- Run from the same directory where `cargo build` would run. If a wrapper builds a nested Cargo
  project, `cd` into that Cargo project or workspace first.
- Prefer `cargo target-gc clean --dry-run` before any deletion.
- Do not run `cargo target-gc clean --confirm` unless the user explicitly approves deletion.
- Do not add `--stale` unless the user asks for broader cleanup or approves after seeing a dry run.
- Do not add `--profile-cache` unless the user asks for stronger cleanup for a huge recent `target/`
  or approves after seeing a dry run.
- Explain that plain `cargo clean` removes the whole target directory, while scoped Cargo clean
  options such as `--package`, `--profile`, `--release`, `--target`, `--target-dir`, and `--doc`
  remove whole selected scopes; target-gc instead uses age/category-based cleanup.
- Never override active Cargo/rustc detection with `--force-active` unless the user explicitly accepts
  the risk.

## Useful Commands

```sh
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --dry-run --profile-cache
cargo target-gc clean --confirm
cargo target-gc config
```

## Suggested Flow

1. Confirm the current directory is the intended Cargo project or workspace.
2. Run `cargo target-gc scan`.
3. Summarize reclaimable bytes, profile cache bytes, and retained/fresh cache.
4. If cleanup is worthwhile, run `cargo target-gc clean --dry-run`.
5. For very large recent targets, offer `cargo target-gc clean --dry-run --profile-cache`.
6. Ask the user before running `cargo target-gc clean --confirm`.
"#;

pub fn parse_hosts(value: &str) -> Result<Vec<HostId>> {
    let mut hosts = Vec::new();
    for raw in value.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let host = HostId::parse(trimmed)
            .ok_or_else(|| anyhow!("unknown host agent {trimmed:?}; supported: claude,codex"))?;
        if !hosts.contains(&host) {
            hosts.push(host);
        }
    }
    if hosts.is_empty() {
        bail!("--only requires at least one host: claude,codex");
    }
    Ok(hosts)
}

pub fn install(opts: &InstallOptions) -> Result<()> {
    let plans = install_plan(opts)?;
    if plans.is_empty() {
        println!("No host agents selected.");
        return Ok(());
    }

    for plan in plans {
        let approved = match opts.selection {
            Selection::All => true,
            Selection::Detected if opts.yes || !opts.only.is_empty() => true,
            Selection::Detected => confirm_host(&plan)?,
        };
        if approved {
            install_one(&plan, opts.collision, opts.dry_run)?;
        } else {
            println!(
                "Skipped {} skill: {}",
                plan.label,
                plan.skill_file.display()
            );
        }
    }
    Ok(())
}

pub fn install_plan(opts: &InstallOptions) -> Result<Vec<PlannedInstall>> {
    let all = [
        host_plan(HostId::Claude, opts.claude_skills_dir.clone())?,
        host_plan(HostId::Codex, opts.codex_skills_dir.clone())?,
    ];

    let selected: Vec<_> = if !opts.only.is_empty() {
        all.into_iter()
            .filter(|p| opts.only.contains(&p.host))
            .collect()
    } else if matches!(opts.selection, Selection::All) {
        all.into_iter().collect()
    } else {
        all.into_iter().filter(|p| p.detected).collect()
    };

    if selected.is_empty() && opts.only.is_empty() && matches!(opts.selection, Selection::Detected)
    {
        println!(
            "No supported host-agent skill directories detected. Re-run with --all or --only claude,codex."
        );
    }

    Ok(selected)
}

fn host_plan(host: HostId, override_dir: Option<PathBuf>) -> Result<PlannedInstall> {
    let (label, config_dir, default_skills_dir) = match host {
        HostId::Claude => (
            "Claude Code",
            home_path(".claude")?,
            override_dir.unwrap_or(home_path(".claude/skills")?),
        ),
        HostId::Codex => (
            "Codex",
            home_path(".codex")?,
            override_dir.unwrap_or(home_path(".codex/skills")?),
        ),
    };
    Ok(PlannedInstall {
        host,
        label,
        detected: config_dir.exists() || default_skills_dir.exists(),
        skill_file: default_skills_dir.join(SKILL_DIR_NAME).join("SKILL.md"),
    })
}

fn home_path(suffix: &str) -> Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home).join(suffix))
}

fn confirm_host(plan: &PlannedInstall) -> Result<bool> {
    if !io::stdin().is_terminal() {
        println!(
            "Detected {} at {}. Non-interactive: skipping (use --yes, --all, or --only).",
            plan.label,
            plan.skill_file.display()
        );
        return Ok(false);
    }
    print!(
        "Install cargo-target-gc skill for {} at {}? [y/N] ",
        plan.label,
        plan.skill_file.display()
    );
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim(), "y" | "Y" | "yes" | "YES"))
}

fn install_one(plan: &PlannedInstall, collision: SkillCollision, dry_run: bool) -> Result<()> {
    if plan.skill_file.exists() {
        match collision {
            SkillCollision::Overwrite => {}
            SkillCollision::Skip => {
                println!(
                    "Keeping existing {} skill: {}",
                    plan.label,
                    plan.skill_file.display()
                );
                return Ok(());
            }
            SkillCollision::Prompt => {
                if !confirm_overwrite(plan)? {
                    println!(
                        "Keeping existing {} skill: {}",
                        plan.label,
                        plan.skill_file.display()
                    );
                    return Ok(());
                }
            }
        }
    }

    if dry_run {
        println!(
            "Would install {} skill: {}",
            plan.label,
            plan.skill_file.display()
        );
        return Ok(());
    }

    if let Some(parent) = plan.skill_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&plan.skill_file, AGENT_SKILL)?;
    println!(
        "Installed {} skill: {}",
        plan.label,
        plan.skill_file.display()
    );
    Ok(())
}

fn confirm_overwrite(plan: &PlannedInstall) -> Result<bool> {
    if !io::stdin().is_terminal() {
        return Ok(false);
    }
    print!(
        "{} skill already exists: {}. Overwrite? [y/N] ",
        plan.label,
        plan.skill_file.display()
    );
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim(), "y" | "Y" | "yes" | "YES"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hosts_accepts_aliases_and_dedupes() {
        let hosts = parse_hosts("claude,cc,codex").expect("parse");
        assert_eq!(hosts, vec![HostId::Claude, HostId::Codex]);
    }

    #[test]
    fn parse_hosts_rejects_unknown_hosts() {
        let err = parse_hosts("gemini").expect_err("unknown host");
        assert!(err.to_string().contains("unknown host agent"));
    }
}
