# cargo-target-gc

**cargo-target-gc** is a Cargo **target-artifact garbage collector**. It discovers a
Cargo project or workspace, analyzes the `target/` directories that builds leave
behind, and reports how much space is **reclaimable**. With an explicit
confirmation it can also clean safe, stale artifacts while preserving build-hot
ones.

`scan` is a pure, read-only filesystem analysis: it **never invokes cargo** and
creates no build artifacts. Run `cargo target-gc` from the same directory where
you would run `cargo build`; it analyzes that Cargo project or workspace's conventional
`target/` directory. Nothing is deleted unless you run `clean` with `--confirm`.

## Where to run cargo target-gc

Use the same working directory you use for Cargo:

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

If a wrapper such as `make` builds a nested Cargo project, cd into that nested
Cargo directory and run `cargo target-gc` there. cargo-target-gc intentionally
does not guess hidden or opaque build targets outside the selected Cargo
project/workspace.

## How artifacts are categorized

A `target/` root is analyzed by a read-only walk (no symlink following) into
four categories:

- **old incremental** — `incremental/` subtrees older than the warm-cache
  window. Cargo regenerates them cheaply, so they are reclaimable.
- **fresh incremental** — recent incremental cache retained for edit-build
  speed (`incremental_retention_hours`).
- **stale** — profile artifacts whose newest modification time is older than the
  retention window (`retention_days`). Reclaimable.
- **retained** — build-hot artifacts within the retention window. Preserved;
  never removed.

Estimated `reclaimable` space is `old incremental + stale`.

## Features

- **Project discovery** — walks up to the nearest `Cargo.toml` and detects
  whether it is a single package or a workspace (enumerating member crates).
- **Target-artifact analysis** — locates every validated `target/` root
  (workspace-shared and per-crate, de-duplicated) and sizes each category.
- **Safe cleaning** — `clean` removes only reclaimable artifacts, only inside a
  validated `target/` root, only with `--confirm`, and refuses when an active
  Cargo/rustc process appears to be using that target unless `--force-active`
  is supplied.
- **Configurable** — an optional `target-gc.toml` sets the retention window and can
  scope analysis to a specific crate path.
- **Normalized output** — human-readable summary by default, or `--json` for
  machine-readable reports (JSON carries raw byte counts).

## Commands

```bash
cargo target-gc scan  [--path <DIR>] [--json]                      # Analyze target/ and report reclaimable space
cargo target-gc clean [--path <DIR>] [--json] --dry-run [--stale]  # Preview removals (deletes nothing)
cargo target-gc clean [--path <DIR>] [--json] --confirm [--stale] [--max-reclaim <SIZE>]
cargo target-gc config [--path <DIR>]                              # Print the effective configuration
cargo target-gc --help                                             # Show usage
```

`scan` resolves a project from `--path` (or the current directory) and prints a
per-root breakdown. `clean` **refuses with a nonzero exit** unless you pass
exactly one of `--dry-run` (preview) or `--confirm` (execute); passing both is
rejected. By default `clean` reclaims only
incremental artifacts; add `--stale` to also reclaim stale ones. Progress is
written to stderr; the report/summary goes to stdout. `clean --confirm` refuses
if an active Cargo/rustc/cargo-watch process appears to be using the selected
target root; stop the build or pass `--force-active` only when you understand the
risk.

If `scan` finds no `target/` directory, run `cargo target-gc` from the Cargo
project or workspace root where `cargo build` creates `target/`.

## Install

From a checkout:

```bash
cargo install --path . --locked
```

After a crates.io release:

```bash
cargo install cargo-target-gc
```

## Configuration (`target-gc.toml`)

```toml
# Artifacts untouched for longer than this many days are considered stale.
retention_days = 14

# Incremental cache newer than this many hours is retained for edit-build speed.
incremental_retention_hours = 24

# Optional safety limit for confirmed clean. The command refuses if planned
# reclaimable bytes exceed this value. CLI --max-reclaim overrides it.
# max_reclaim_bytes = 1073741824

# Optional: scope analysis to a specific crate within the project. Must be a
# relative path that stays inside the project root; absolute paths or parent
# traversal that escapes the project are rejected.
# crate_path = "crates/core"
```

A missing `target-gc.toml` falls back to the defaults shown above
(`retention_days = 14`, `incremental_retention_hours = 24`, no reclaim limit,
no crate scope).

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
├── config.rs      # target-gc.toml loading (retention + crate scope)
├── target.rs      # Read-only target/ analysis + categorization
├── scan.rs        # Scan orchestration → report
├── clean.rs       # Reclaimable-artifact removal (dry-run / confirm)
└── report.rs      # Report model + text/JSON rendering
tests/
├── cli.rs         # End-to-end CLI tests
└── fixtures/      # Sample projects for discovery tests
docs/               # Architecture, plans, patterns, specs
```

## Open Source

- License: `MIT OR Apache-2.0`; see [LICENSE](LICENSE).
- Contributions: see [CONTRIBUTING.md](CONTRIBUTING.md).
- Security-sensitive cleanup bugs: see [SECURITY.md](SECURITY.md).
- Release notes: see [CHANGELOG.md](CHANGELOG.md).
- Maintainer release process: see [RELEASE.md](RELEASE.md).
