# Open Source Readiness

This document records the recommended public-release defaults for cargo-target-gc.

## Task Evaluation

Verdict: `ACCEPT` after adding the missing maintainer release guide.

Scope: repository docs, GitHub templates, CI, and Cargo package metadata.

Primary risk: cargo-target-gc can delete files with `clean --confirm`, so public docs
must keep the safety model more prominent than marketing language.

## Positioning

Best call: present cargo-target-gc as a Cargo `target/` garbage collector, not a general
Rust project fixer. The promise is narrow and safety-focused: report reclaimable
target artifacts and clean them only with explicit confirmation.

## License

Best call: use `MIT OR Apache-2.0`. This matches common Rust ecosystem practice,
keeps downstream adoption easy, and is reflected in `Cargo.toml`.

## Release Stage

Best call: publish the first public version as `0.1.0` with beta language in the
README and release notes. Avoid claiming production-grade status until more real
workspace testing and user reports exist.

## Safety Policy

Best call: treat unsafe deletion as security-sensitive. `scan` must remain
read-only, `clean` must require `--confirm`, and deletion paths must stay inside
validated Cargo `target/` roots.

## CI

Best call: run format, lint, tests, and build on Linux and macOS. Filesystem and
process behavior differ enough that macOS coverage is useful for this tool.

## Contributions

Best call: require PRs to describe behavior changes, verification commands, and
any deletion-safety impact. Most valuable early contributions are realistic
workspace fixtures, edge-case tests, and documentation improvements.

## Publishing

Best call before crates.io publication:

1. Confirm the public GitHub repository is available.
2. Confirm repository, homepage, and documentation URL metadata in `Cargo.toml`.
3. Run CI successfully on the default branch.
4. Tag `v0.1.0`.
5. Publish with `cargo publish --dry-run`, then `cargo publish`.

See `RELEASE.md` for the maintainer checklist.

## Decision Checklist

| Item | Best call | Status |
|------|-----------|--------|
| Product promise | Cargo `target/` artifact garbage collector | Documented |
| License | `MIT OR Apache-2.0` | Documented and set in Cargo metadata |
| README | Install, quick start, safety model, config, project structure | Present |
| Changelog | Keep a Changelog style with `Unreleased` section | Present |
| Contributing | Require verification commands and deletion-safety notes | Present |
| Security | Treat unsafe cleanup as private vulnerability reports | Present |
| Code of conduct | Lightweight maintainer-enforced policy | Present |
| Issue templates | Bug, feature request, unsafe cleanup report | Present |
| PR template | Summary, verification, safety impact | Present |
| CI | Linux and macOS fmt, clippy, test, build | Present |
| Release process | `0.1.0` safe beta, semver, crates.io checklist | Present |
| Cargo repository URL | Public GitHub/docs URLs in Cargo metadata | Present |

## Acceptance Criteria

- Public docs explain where to run cargo-target-gc and what it may delete.
- Maintainers have release and rollback instructions.
- Contributors have setup, test, PR, and safety expectations.
- CI runs the same core checks documented for contributors.
- crates.io metadata includes public repository, homepage, and documentation
  URLs.
