# Contributing to cargo-target-gc

Thanks for helping improve cargo-target-gc. The project is intentionally conservative:
`scan` must be read-only, and `clean` must delete only reclaimable artifacts
inside validated Cargo `target/` directories.

## Development Setup

Install a recent stable Rust toolchain, then work from the repository root:

```bash
make build
make test
make lint
make fmt-check
```

For local manual testing, use a disposable Cargo project or run
`cargo target-gc clean --dry-run` before any confirmed clean.

## Design Rules

- Run `cargo target-gc` from the directory where `cargo build` would run.
- Keep target discovery explicit; do not guess hidden wrapper build outputs.
- Keep `src/target.rs` as the single place that walks and categorizes target
  artifacts.
- Keep `clean` backed by the same analysis used by `scan`.
- Preserve fresh incremental cache by default for edit-build performance.
- Add tests for every behavior change that affects scan output, deletion
  planning, config parsing, or CLI flags.

## Pull Requests

Before opening a pull request:

1. Describe the user-visible behavior change.
2. Include the commands you ran for verification.
3. Link related issues when available.
4. Include terminal output or screenshots for CLI output changes.
5. Call out any safety impact, especially changes to deletion scope.

Use concise imperative commit messages, for example `Add clean limit guard` or
`Document target root expectations`.
