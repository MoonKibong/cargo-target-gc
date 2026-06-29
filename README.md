# derust

**derust** is a Cargo **target-artifact garbage collector**. It discovers a
Cargo project or workspace, analyzes the `target/` directories that builds leave
behind, and reports how much space is **reclaimable**. With an explicit
confirmation it can also clean safe, stale artifacts while preserving build-hot
ones.

`scan` is a pure, read-only filesystem analysis: it **never invokes cargo** and
creates no build artifacts. It is safe to run anywhere — it only reads file
sizes and modification times under `target/`. Nothing is deleted unless you run
`clean` with `--confirm`.

## How artifacts are categorized

A `target/` root is analyzed by a read-only walk (no symlink following) into
three categories:

- **incremental** — `incremental/` subtrees. Always reclaimable; cargo
  regenerates them cheaply.
- **stale** — profile artifacts whose newest modification time is older than the
  retention window (`retention_days`). Reclaimable.
- **retained** — build-hot artifacts within the retention window. Preserved;
  never removed.

Estimated `reclaimable` space is `incremental + stale`.

## Features

- **Project discovery** — walks up to the nearest `Cargo.toml` and detects
  whether it is a single package or a workspace (enumerating member crates).
- **Target-artifact analysis** — locates every validated `target/` root
  (workspace-shared and per-crate, de-duplicated) and sizes each category.
- **Safe cleaning** — `clean` removes only reclaimable artifacts, only inside a
  validated `target/` root, and only with `--confirm`.
- **Configurable** — an optional `derust.toml` sets the retention window and can
  scope analysis to a specific crate path.
- **Normalized output** — human-readable summary by default, or `--json` for
  machine-readable reports (JSON carries raw byte counts).

## Commands

```bash
derust scan  [--path <DIR>] [--json]                      # Analyze target/ and report reclaimable space
derust clean [--path <DIR>] [--json] --dry-run [--stale]  # Preview removals (deletes nothing)
derust clean [--path <DIR>] [--json] --confirm [--stale]  # Remove reclaimable artifacts
derust config [--path <DIR>]                              # Print the effective configuration
derust --help                                             # Show usage
```

`scan` resolves a project from `--path` (or the current directory) and prints a
per-root breakdown. `clean` **refuses with a nonzero exit** unless you pass
exactly one of `--dry-run` (preview) or `--confirm` (execute); passing both is
rejected. By default `clean` reclaims only
incremental artifacts; add `--stale` to also reclaim stale ones. Progress is
written to stderr; the report/summary goes to stdout.

## Configuration (`derust.toml`)

```toml
# Artifacts untouched for longer than this many days are considered stale.
retention_days = 14

# Optional: scope analysis to a specific crate within the project. Must be a
# relative path that stays inside the project root; absolute paths or parent
# traversal that escapes the project are rejected.
# crate_path = "crates/core"
```

A missing `derust.toml` falls back to the defaults shown above
(`retention_days = 14`, no crate scope).

## Build

```bash
make build    # cargo build
make test     # cargo test
make lint     # cargo clippy -- -D warnings
make fmt      # cargo fmt
```

## Project Structure

```
src/
├── main.rs        # CLI entry point + command dispatch
├── lib.rs         # Library root / module declarations
├── cli.rs         # clap argument definitions
├── discovery.rs   # Cargo project / workspace discovery
├── config.rs      # derust.toml loading (retention + crate scope)
├── target.rs      # Read-only target/ analysis + categorization
├── scan.rs        # Scan orchestration → report
├── clean.rs       # Reclaimable-artifact removal (dry-run / confirm)
└── report.rs      # Report model + text/JSON rendering
tests/
├── cli.rs         # End-to-end CLI tests
└── fixtures/      # Sample projects for discovery tests
docs/               # Architecture, plans, patterns, specs
```

## Contributing

See [CLAUDE.md](CLAUDE.md) for coding conventions and workflow.
