# CLAUDE.md

This file guides Claude Code and Codex when working in **derust**.

> Keep under 150 lines: rules and links only. Everything else in `docs/`.

## What This Is

derust — a read-only Rust project health and refactoring-readiness CLI. It
discovers a Cargo project/workspace, runs read-only toolchain checks (`cargo
check`/`test`/`fmt --check`/`clippy -- -D warnings`), and reports a normalized
human-readable or JSON summary. Commands: `scan`, `config`.

Tech: Rust (edition 2021); clap (derive), serde/serde_json, toml, anyhow;
assert_cmd + predicates for CLI tests.

## Priority Guide

**ALWAYS ENFORCE:**
1. Read-only by default — derust must never modify a target project. No auto-fix
   in the MVP (clippy runs `-- -D warnings`, fmt runs `--check`). Auto-fix is
   documented future work only.
2. Small, modular files with one shared cargo runner (no per-check copy-paste);
   no `unwrap()`/`expect()` outside tests — handle errors with typed/`anyhow`
   results.

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
