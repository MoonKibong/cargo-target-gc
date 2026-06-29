# Repository Guidelines

## What derust Is

derust is a Cargo **target-artifact garbage collector**: it analyzes `target/` directories, reports **reclaimable** space, and (only with explicit confirmation) cleans safe/stale build artifacts while preserving build-hot ones. `scan` is a pure, read-only filesystem analysis — it never invokes cargo and creates no build artifacts; `clean` is the only mutating command and refuses without `--dry-run`/`--confirm`. Centralize the `target/` walk in `src/target.rs`; `clean` reuses `target::analyze`. No `unwrap()`/`expect()` outside tests.

## Project Structure & Module Organization

Source lives in `src/` (`main.rs`, `lib.rs`, `cli.rs`, `discovery.rs`, `config.rs`, `target.rs`, `scan.rs`, `clean.rs`, `report.rs`), tests in `tests/`, and project documentation under `docs/` (`architecture/`, `implementation/`, `patterns/`, `specs/`, `schemas/`, `dev/`, and `archive/implementation/`). `Makefile`, `README.md`, and `CLAUDE.md` define the build commands and conventions. Keep the source layout simple and predictable; put developer scripts in `scripts/`.

Prefer small modules with clear ownership. For example, Rust code should use `src/lib.rs` for reusable logic and `src/main.rs` for CLI or executable entry points. Name files and directories after their domain purpose, such as `parser`, `commands`, or `fixtures`.

## Build, Test, and Development Commands

The canonical commands are defined in the `Makefile` and should be run from the repository root. Keep the same target names so the documented workflow stays stable.

- `make build`: `cargo build`.
- `make test`: `cargo test`.
- `make lint`: `cargo clippy -- -D warnings`.
- `make fmt`: `cargo fmt` (`make fmt-check` for `cargo fmt --check`).

## Coding Style & Naming Conventions

Follow the formatter for the language in use rather than hand-formatting. For Rust, use `rustfmt` defaults and prefer `snake_case` for functions, modules, and variables, `PascalCase` for types and traits, and `SCREAMING_SNAKE_CASE` for constants.

Keep public APIs small and documented. Avoid hardcoded absolute paths; prefer configuration, arguments, or test fixtures under `tests/fixtures/`.

## Testing Guidelines

Add tests with each behavioral change. Use unit tests close to the code they cover and integration tests under `tests/` for externally visible behavior. Name tests after the behavior being verified, for example `parses_empty_input` or `rejects_invalid_config`.

Include fixture files only when they make tests clearer. Keep fixtures minimal and document any intentionally unusual inputs.

## Commit & Pull Request Guidelines

No repository Git history is available yet, so use concise imperative commit messages such as `Add parser tests` or `Fix config validation`. Keep each commit focused on one logical change.

Pull requests should include a short summary, the commands run for verification, linked issues when applicable, and screenshots or terminal output for user-facing changes. Call out follow-up work explicitly rather than hiding it in code comments.

## Agent-Specific Instructions

Before modifying repository conventions, inspect existing files first and preserve user changes. If shell environment variables or PATH entries are needed, add exports only to `~/.config/shell/env.sh`; do not edit `.zprofile`, `.bash_profile`, `.bashrc`, `.profile`, or `.zshenv` for that purpose.
