---
order: cargo-target-gc
updated: 2026-06-29T09:01:33Z
status: implemented
---

# Plan: cargo-target-gc product pivot → Cargo target artifact GC

## Context & recognition
cargo-target-gc is being repurposed from a "read-only Rust health-check scanner" (runs cargo
check/test/fmt/clippy via probe.rs + Sandbox) into a **Cargo target-artifact garbage
collector**: it analyzes `target/` directories, reports reclaimable space, and can
clean safe/stale artifacts while preserving build-hot artifacts. `scan` must become a
pure filesystem analysis that NEVER invokes cargo or creates build artifacts.

Prior failure modes: `docs/dev/pipeline-hardening-notes.md` does not exist — no recorded
entries for this area.

Current code: src/{cli,config,discovery,probe,report,scan,main,lib}.rs (~2000 LoC,
largest probe.rs 273). discovery.rs is reusable as-is. probe.rs (cargo runner + Sandbox)
becomes obsolete. config.rs/report.rs/scan.rs/cli.rs/main.rs must be repurposed.

## Domain model (the contract Task 1 documents, Task 2 implements)
- **TargetRoot**: a `target/` dir belonging to the discovered project/workspace (dedup
  shared workspace target + any per-member targets). Validated as a real cargo target
  (basename `target` adjacent to a Cargo.toml, or contains `CACHEDIR.TAG`).
- **Category** of artifacts within a root, by read-only walk (no symlink follow, no cargo):
  - `Incremental` — `incremental/` subtrees → always reclaimable (regenerated cheaply).
  - `Stale` — profile artifacts whose newest mtime is older than `retention_days` → reclaimable.
  - `Retained` (hot) — artifacts within the retention window → preserved (NOT reclaimable).
- Per root: total_bytes, per-category bytes, estimated reclaimable_bytes = Incremental + Stale.
- Progress → stderr; final report → stdout (text) or stdout JSON with `--json`.

## T1 — Documentation & spec pivot (durable docs)
Files: NEW `docs/implementation/CARGO_TARGET_GC_PLAN.md`; update `README.md`,
`CLAUDE.md`, `AGENTS.md`, and `Cargo.toml` `description`.
- Plan doc records the domain model above + CLI semantics: `scan` (read-only analysis),
  `clean --dry-run` (preview), `clean --confirm` (execute, safe categories only),
  refusal when neither flag given; stderr-progress/stdout-report rule; explicit
  "scan never runs cargo check/test/fmt/clippy and creates no build artifacts".
- README/CLAUDE/AGENTS restated to the GC purpose; remove claims that scan runs
  cargo health checks; keep read-only-by-default and no-unwrap rules.
- Reference workflow gate: executor creates a `plan` issue (P#) and `task` issue (T#)
  and puts `plan_id: P#N` in the plan doc frontmatter; commits reference T#.
Acceptance: `grep -ri "garbage collect\|target artifact\|reclaimable" README.md CLAUDE.md AGENTS.md`
hits all three; no surviving "runs cargo check/test/fmt/clippy" health-check description;
Cargo.toml description mentions target GC; plan doc present with model + CLI semantics.

## T2 — Core pivot: scan = read-only target artifact analysis (atomic, keeps build green)
- NEW `src/target.rs`: `TargetRoot`, `Category`, `ArtifactGroup`; `locate_roots(project)`
  (dedup, target-dir validation) and `analyze(root, retention)` that walks read-only
  (std::fs + symlink_metadata, no follow, no Command/cargo) summing sizes and splitting
  Incremental/Stale/Retained by mtime vs `retention_days`. Unit-tested with a temp fixture
  tree using `std::fs::File::set_modified` (stable since 1.75) to set deterministic mtimes.
- Rewrite `src/config.rs`: replace `Checks` toggles with GC config —
  `retention_days: u64` (default 14) + optional `crate_path` scope; update unit tests.
- Rewrite `src/report.rs`: new model `ScanReport { roots: Vec<TargetRootReport>, summary }`
  where TargetRootReport has path, total_bytes, per-category bytes, reclaimable_bytes;
  text render (human-readable sizes) + JSON (raw bytes); keep json-round-trip + text tests.
- Rewrite `src/scan.rs`: discovery → locate_roots → analyze → build_report; progress to stderr.
- DELETE `src/probe.rs`; update `src/lib.rs` (remove `pub mod probe;`, add `pub mod target;`).
- Update `src/main.rs` + `src/cli.rs`: keep `scan --path --json`; `config` prints GC fields;
  report → stdout, progress → stderr (preserve/merge any existing uncommitted stderr-progress edits).
- Update `tests/fixtures` (add a fake `target/` tree: debug/incremental, debug/deps, etc.)
  and rewrite `tests/cli.rs` scan/config assertions for the new output.
Acceptance: `cargo build` + `cargo test` green; scan output (text+JSON) shows roots,
sizes, categories, reclaimable_bytes; a test asserts scan spawns no cargo and leaves the
fixture `target/` byte-identical and creates no new `target/`/`Cargo.lock`; JSON has
`roots` and `reclaimable_bytes` keys. Reuse: scan and (later) clean both call
`target::analyze` — no duplicated walk.

## T3 — `clean` command: dry-run + confirmed execution, safe categories only
- NEW `src/clean.rs`: compute deletion set by REUSING `target::analyze` (reclaimable
  categories only — Incremental by default, Stale via `--stale`/`--max-age`); hard safety
  guard that every deletion path lies inside a validated cargo `target/` root; `--dry-run`
  lists planned removals + bytes and deletes nothing; execution removes only with
  `--confirm`; with neither flag, refuse with guidance and nonzero exit. Progress→stderr,
  summary→stdout/JSON.
- `src/cli.rs`: add `Clean { path, json, dry_run, confirm, stale (or max_age) }`.
  `src/main.rs`: dispatch clean; progress on stderr. `src/lib.rs`: `pub mod clean;`.
- Tests (unit + `tests/cli.rs`): dry-run leaves fixture byte-identical; `--confirm` removes
  only Incremental/Stale and preserves Retained/hot; refusal (nonzero, message) when neither
  `--dry-run` nor `--confirm`; refusal/guard when target path is not a cargo target dir.
Acceptance: `cargo test` covers all four behaviors above; clean never touches Retained
artifacts; CLI flags/semantics match the doc from T1.

## T4 — Verification & doc-consistency
Run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `make build`; fix
any failures. Confirm README/CLAUDE/AGENTS CLI matches implemented `scan`/`clean` flags;
update docs if drift. Confirm no `unwrap()`/`expect()` outside `#[cfg(test)]`.
Acceptance: all four commands exit 0; `grep -rn "unwrap()\|expect(" src | grep -v test`
shows none in production paths; doc CLI surface == actual `--help`.

## Dependency order
T1 → T2 → T3 → T4 (single repo: /repo/root). T2 must land atomically
(removing probe.rs while rewriting scan/config/report) so the tree always compiles. T3
builds on T2's `target.rs`. T4 is the final gate.

## Risks & mitigations
- **Destructive clean**: path validation (must be inside a verified cargo `target/`),
  `--confirm` gate, dry-run default-safe, and tests asserting Retained/non-target paths
  are never deleted.
- **mtime-flaky tests**: set mtimes deterministically via `std::fs::File::set_modified`
  (no new dependency).
- **Symlink escape / cycles**: use `symlink_metadata`, never follow symlinked dirs.
- **Workspace shared vs per-crate target**: dedup roots in `locate_roots`.
- **Scope creep**: no new crates needed; manual std::fs walk avoids adding `walkdir`.

## Quality gates
- No file projected >1000 LoC (largest new `target.rs` ~250) — no split task needed.
- Walk/categorization centralized in `target.rs`; `clean.rs` reuses it (no copy-paste).
- Every task's acceptance is diffable/testable via cargo commands or grep.


