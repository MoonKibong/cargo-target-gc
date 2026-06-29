# CLAUDE.md

This file guides Claude Code and Codex when working in **cargo-target-gc**.

> Keep under 150 lines: rules and links only. Everything else in `docs/`.

## What This Is

cargo-target-gc — a Cargo **target-artifact garbage collector**. It discovers a Cargo
project/workspace, analyzes the `target/` directories left by builds, and
reports reclaimable space; with confirmation it cleans safe/stale artifacts
while preserving build-hot ones. Commands: `scan` (read-only analysis), `clean`
(`--dry-run` / `--confirm`), `config`.

`scan` is a pure filesystem analysis: it **never invokes cargo** and creates no
build artifacts. Artifacts are categorized as **incremental** (older than
`incremental_retention_hours` → reclaimable), **stale** (older than
`retention_days` → reclaimable), or **retained** (build-hot/warm → preserved).
Reclaimable = old incremental + stale.

Run `cargo target-gc` from the same directory where `cargo build` would be run. If a wrapper
such as `make` builds a nested Cargo project, the user should `cd` into that
Cargo project/workspace before running `cargo target-gc`; cargo-target-gc must not guess opaque
wrapper build paths.

Tech: Rust (edition 2021); clap (derive), serde/serde_json, toml, anyhow;
assert_cmd + predicates for CLI tests.

## Priority Guide

**ALWAYS ENFORCE:**
1. Read-only by default — `scan` must never modify a target project and must
   never run cargo. Scope is explicit: analyze the selected Cargo root's
   conventional `target/` directory, not hidden/nested wrapper targets. `clean`
   is the only mutating command: it refuses without `--dry-run`/`--confirm`,
   refuses active target roots unless explicitly forced, removes only
   reclaimable artifacts, and only inside a validated `target/` root.
2. Centralize the `target/` walk/categorization in `target.rs`; `clean` reuses
   `target::analyze` (no duplicated walk). No `unwrap()`/`expect()` outside
   tests — handle errors with typed/`anyhow` results.

**DATA SAFETY:**
- Never store secrets, tokens, or credentials in source files or logs.

**PREFER:**
- Small, reviewable changes; one coherent task per commit.

## Commands

```
make build    # Build the project
make test     # Run tests
make lint     # Run linter
make fmt      # Auto-format code
```

Single test: `cargo test <name>` (e.g. `cargo test discovery`)

## Documentation Map

| Topic | Location |
|-------|----------|
| Architecture | `docs/architecture/ARCHITECTURE.md` |
| Open source readiness | `docs/oss/OPEN_SOURCE_READINESS.md` |
| Contributing | `CONTRIBUTING.md` |
| Security policy | `SECURITY.md` |
| Implementation plans | `docs/implementation/` |
| Reusable patterns | `docs/patterns/` |
| Context engineering | `docs/dev/` |
| Workflow-gate boilerplate | `docs/dev/global-claude-md.template` |
| Archived plans | `docs/archive/implementation/_INDEX.md` |

## Workflow Gates

**BEFORE creating a plan doc** (`docs/implementation/*_PLAN.md`):
1. `gh issue create --label plan --title "{title}"` → get `P#N`
2. Put `plan_id: P#N` in the plan doc frontmatter
3. Run `/task-evaluate` before implementation

**BEFORE starting implementation**:
1. `gh issue create --label task --title "{title}"` → get `T#N`
2. Run `/task-execute` for the implementation
3. Reference `T#N` in commits: `feat(T#N): description`

## Harness Skills

- `/task-evaluate`: ANY plan, spec, or design doc before implementation.
- `/task-execute`: ANY non-trivial implementation work.
- Skip only for trivial operations (typo fixes, commit/push, file reads, questions).
