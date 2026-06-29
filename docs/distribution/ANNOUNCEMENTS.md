# Announcement Copy

Use these as starting points for community posts. Keep the tone practical and
avoid overstating safety: `clean --confirm` deletes files.

## Short Post

I built cargo-target-gc, a dry-run-first Cargo `target/` cleanup tool that
reclaims Rust build artifacts by age/category instead of deleting the whole
target directory.

## Rust Users Forum / Reddit

Title:

```text
cargo-target-gc: dry-run-first cleanup for large Cargo target directories
```

Body:

```markdown
I published `cargo-target-gc`, a Cargo subcommand for Rust projects whose
`target/` directories grow large.

The motivation was practical: when I build with coding agents like Claude Code,
Codex, and Gemini CLI, the build/test/retry loop can make `target/` grow much
faster than I notice. `cargo clean` is useful, and it has scoped options like
`--package`, `--profile`, and `--target`, but those still clean whole selected
scopes. I wanted a more conservative workflow:

- `cargo target-gc scan`
- inspect old incremental, fresh incremental, stale, profile cache, and retained
  artifacts
- run `cargo target-gc clean --dry-run`
- only delete with `--confirm`

For very large recent targets there is an explicit stronger mode,
`--profile-cache`, but the default keeps warm cache value.

Install:

```bash
cargo install cargo-target-gc --locked
```

Repo: https://github.com/MoonKibong/cargo-target-gc
Crate: https://crates.io/crates/cargo-target-gc
```

## Show HN

```text
Show HN: cargo-target-gc, a dry-run-first cleaner for Rust target directories
```

```markdown
I built cargo-target-gc after seeing Rust `target/` directories grow very large
during agentic coding sessions.

It is a Cargo subcommand that scans first, categorizes artifacts by cache role
and age, and only deletes after explicit confirmation. It is not trying to
replace `cargo clean`; Cargo's scoped clean options are still useful. The
difference is that target-gc reports and cleans old incremental/stale artifacts
while preserving warm cache by default.

https://github.com/MoonKibong/cargo-target-gc
```

## LinkedIn / Facebook

```text
I published cargo-target-gc, a Rust tool that helps reclaim Cargo target/ disk
space safely with scan-first, dry-run-first cleanup.
```
