---
order: cargo-target-gc-mvp-scan
updated: 2026-06-29T08:05:07Z
---

# Plan: cargo-target-gc-mvp-scan

---
order: cargo-target-gc-mvp-scan
updated: 2026-06-29T08:30:00Z
---

# Plan: cargo-target-gc-mvp-scan

# cargo-target-gc MVP — Read-only Rust Project Health Scan CLI

## Context & Decisions
- Greenfield Rust scaffold: no Cargo.toml/src yet. Toolchain: cargo/rustc 1.94, clippy+fmt present.
- No docs/dev/pipeline-hardening-notes.md exists (nothing to reference). docs/patterns/RUST_MODULE_PATTERNS.md absent; MVP keeps every file small/modular so the >1000-line split gate does not trigger.
- Stack: clap(derive), serde, serde_json, toml, anyhow; dev-dep assert_cmd + predicates for CLI tests.
- Read-only ONLY: clippy runs `-- -D warnings`, fmt runs `--check`; NO auto-fix. Auto-fix documented as future work only.
- DEDUP: a single generic cargo-command runner in probe.rs is shared by all four checks (check/test/fmt/clippy) — no per-check copy-paste.
- Flag to PM: target-gc.toml schema kept minimal for MVP (check toggles + optional crate path); not a blocker.

### Task 1 — Cargo skeleton, CLI arg layer, Makefile, docs
Files: Cargo.toml, src/main.rs, src/lib.rs, src/cli.rs, Makefile, README.md, CLAUDE.md.
- Cargo.toml: package `cargo-target-gc` (edition 2021), bin `cargo-target-gc`, deps clap(derive)/serde/serde_json/toml/anyhow, dev-deps assert_cmd/predicates.
- src/cli.rs: clap `Cli` with subcommands `Scan{ path: Option<PathBuf>, json: bool }` and `Config{ path: Option<PathBuf> }`.
- src/lib.rs declares `pub mod cli;` (+ later modules); src/main.rs parses Cli and dispatches to stub handlers returning Ok.
- Makefile: build→`cargo build`, test→`cargo test`, lint→`cargo clippy -- -D warnings`, fmt→`cargo fmt`.
- README.md + CLAUDE.md: replace TODO placeholders — define cargo-target-gc as a read-only Rust project health & refactoring-readiness CLI; list scan/config commands and feature scope; add "Future work: auto-fix mode (not in MVP)".
Acceptance: `cargo build` exits 0; `cargo run -- --help` stdout contains `scan` and `config`; `cargo run -- scan --help` stdout contains `--json` and `--path`; `make build` invokes `cargo build`, `make lint` invokes `cargo clippy -- -D warnings`, `make fmt` invokes `cargo fmt`; `grep -c '\[TODO' README.md CLAUDE.md` returns 0 matches.

### Task 2 — Project/workspace discovery (src/discovery.rs)
- Resolve a root by walking up from given/cwd path to nearest Cargo.toml; parse to detect `[workspace]` vs `[package]`; enumerate member crate manifests for workspaces.
- Add `pub mod discovery;` to src/lib.rs.
- Create tests/fixtures/ minimal sample projects (single-package, workspace-with-2-members, and a no-manifest dir).
Acceptance: unit tests in `src/discovery.rs` cover single-package (asserts one crate found), workspace-with-2-members (`assert_eq!` member count == 2), and no-Cargo.toml (returns typed `Err`); `cargo test discovery` exits 0.

### Task 3 — Config loading + `config` subcommand (src/config.rs)
- serde `Config` struct for optional target-gc.toml: per-check enable toggles (check/test/fmt/clippy) with sensible defaults, optional target crate path.
- Missing file → defaults; invalid toml → typed error (no panic).
- Add `pub mod config;` to src/lib.rs; wire main.rs `Config` subcommand to load + print resolved config.
Acceptance: unit tests in `src/config.rs` cover missing-file→`Config::default()`, valid-file→parsed toggle values (`assert_eq!`), invalid-toml→`Err` (no panic); `cargo run -- config` prints the effective config to stdout; `cargo test config` exits 0.

### Task 4 — Toolchain probe + shared cargo runner (src/probe.rs)
- Probe availability of cargo subcommands (check/test/clippy/fmt) via `cargo <x> --version`/`which`.
- Single generic `fn run_check(cmd_args) -> CheckResult` capturing status/stdout/stderr; map to `CheckStatus { Ok, Failed, Skipped, Unavailable }`. Commands are read-only: clippy `-- -D warnings`, fmt `--check`.
- Add `pub mod probe;` to src/lib.rs.
Acceptance: unit tests in `src/probe.rs` assert exit code 0 → `CheckStatus::Ok`, nonzero → `CheckStatus::Failed`, and the unavailable/skipped mapping (no real cargo invocation needed); `cargo test probe` exits 0.

### Task 5 — Scan orchestration + report rendering (src/scan.rs, src/report.rs)
- scan.rs: discovery → config → probe → run enabled checks via probe runner → build `ScanReport` (per-check status + summary).
- report.rs: human-readable normalized summary; `--json` emits serde_json `ScanReport`.
- Add `pub mod scan; pub mod report;` to src/lib.rs; wire main.rs `Scan` subcommand (honors `--path`/`--json`).
Acceptance: `cargo run -- scan --path tests/fixtures/single-package` prints a normalized per-check report; `cargo run -- scan --path tests/fixtures/single-package --json` emits JSON that parses via `serde_json::from_str` into a per-check array; a unit test in `src/report.rs` asserts `serde_json::to_string(&ScanReport)` succeeds and round-trips; `cargo test scan report` exits 0.

### Task 6 — CLI integration tests + full verification
Files: tests/cli.rs.
- assert_cmd tests: `--help` lists subcommands; `scan` on fixture exits/report; `scan --json` output parses as JSON; `config` prints config; nonexistent path → nonzero exit with error message.
- Final verification: `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`, and `make build`/`make test`/`make lint` all green.
Acceptance: `tests/cli.rs` assert_cmd cases pass — `--help` stdout contains `scan`/`config`, `scan --json` on `tests/fixtures/single-package` parses as JSON, `config` prints config, nonexistent path exits nonzero with a stderr error message; `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings` all exit 0 with zero failures/warnings.

