# derust

**derust** is a read-only Rust project health and refactoring-readiness CLI. It
discovers a Cargo project or workspace, runs a set of read-only toolchain checks,
and reports a normalized summary of how healthy and refactor-ready the project is.

derust never modifies your code. It is safe to run in CI or on an unfamiliar
repository: clippy runs with `-- -D warnings` and formatting is checked with
`cargo fmt --check`, but nothing is ever rewritten.

## Features

- **Project discovery** — walks up to the nearest `Cargo.toml` and detects whether
  it is a single package or a workspace (enumerating member crates).
- **Read-only health checks** — wraps `cargo check`, `cargo test`,
  `cargo fmt --check`, and `cargo clippy -- -D warnings`.
- **Configurable** — an optional `derust.toml` toggles individual checks and can
  target a specific crate path.
- **Normalized output** — human-readable summary by default, or `--json` for
  machine-readable reports.

### Future work

- **Auto-fix mode** (apply `cargo fmt` / `cargo fix` / clippy suggestions) is
  intentionally **not** part of this MVP. derust stays strictly read-only for now;
  an opt-in fix mode is planned as future work.

## Commands

```bash
derust scan [--path <DIR>] [--json]   # Run read-only health checks and report
derust config [--path <DIR>]          # Print the effective configuration
derust --help                         # Show usage
```

`scan` resolves a project from `--path` (or the current directory), runs the
enabled checks, and prints a per-check report. `config` shows how `derust.toml`
(if present) resolves against the built-in defaults.

## Configuration (`derust.toml`)

```toml
# Optional: scope checks to a specific crate within the project.
# Must be a root-level key, declared before the [checks] table.
# crate_path = "crates/core"

[checks]
check = true    # cargo check
test = true     # cargo test
fmt = true      # cargo fmt --check
clippy = true   # cargo clippy -- -D warnings
```

A missing `derust.toml` falls back to the defaults shown above (all checks on).

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
├── config.rs      # derust.toml loading
├── probe.rs       # Shared read-only cargo command runner
├── scan.rs        # Scan orchestration
└── report.rs      # Report model + text/JSON rendering
tests/
├── cli.rs         # End-to-end CLI tests
└── fixtures/      # Sample projects for discovery/scan tests
docs/               # Architecture, plans, patterns, specs
```

## Contributing

See [CLAUDE.md](CLAUDE.md) for coding conventions and workflow.
