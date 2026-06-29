# Repository Guidelines

## Project Structure & Module Organization

The repository is being scaffolded: `Makefile`, `README.md`, and `CLAUDE.md` define the current conventions, and project documentation lives under `docs/` (`architecture/`, `implementation/`, `patterns/`, `specs/`, `schemas/`, `dev/`, and `archive/implementation/`). Keep the source layout simple and predictable as code is added: place application source in `src/`, tests in `tests/`, and static or sample assets in `assets/`. Put developer scripts in `scripts/`.

Prefer small modules with clear ownership. For example, Rust code should use `src/lib.rs` for reusable logic and `src/main.rs` for CLI or executable entry points. Name files and directories after their domain purpose, such as `parser`, `commands`, or `fixtures`.

## Build, Test, and Development Commands

The canonical commands are defined in the `Makefile` and should be run from the repository root. These targets are placeholders today and currently shell out to `echo`; flesh them out as the build is implemented, keeping the same target names so the documented workflow stays stable.

- `make build`: build the project.
- `make test`: run the full test suite.
- `make lint`: run lint checks.
- `make fmt`: auto-format code.

For a Rust project, wire these targets to the underlying tools — for example `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt`.

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
