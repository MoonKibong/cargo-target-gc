---
title: Architecture
status: draft
---

# Architecture: cargo-target-gc

cargo-target-gc is a conservative Cargo `target/` artifact garbage collector. It is
organized around one safety rule: analysis and deletion must agree on the same
validated target roots and artifact categories.

## Command Flow

- `src/main.rs` dispatches CLI commands and handles user-facing errors.
- `src/cli.rs` defines flags and help text with `clap`.
- `scan` resolves the selected Cargo project, loads config, analyzes target
  roots, and renders text or JSON.
- `clean` builds a plan from the same analysis used by `scan`; execution happens
  only when `--confirm` is supplied.

## Discovery and Scope

`src/discovery.rs` walks upward to a `Cargo.toml` and determines whether the
project is a package or workspace. cargo-target-gc expects to run from the same directory
where `cargo build` would run. It intentionally does not infer target roots from
wrapper commands such as `make`.

## Target Analysis

`src/target.rs` owns the filesystem walk. It validates Cargo `target/` roots,
does not follow symlinks, sizes artifacts, and categorizes them as old
incremental, fresh incremental, stale, or retained. Fresh incremental artifacts
are retained by default to protect edit-build performance.

## Cleaning

`src/clean.rs` converts reclaimable artifacts into a deletion plan. Before
removing anything it re-validates paths, checks active Cargo/rustc processes,
and enforces configured or CLI reclaim limits.

## Reporting

`src/report.rs` is the stable reporting boundary. Text output is optimized for
humans; JSON output carries raw byte counts for automation.
