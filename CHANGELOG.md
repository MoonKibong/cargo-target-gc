# Changelog

All notable changes to cargo-target-gc will be documented in this file.

The format follows Keep a Changelog, and this project uses semantic versioning
once public releases begin.

## [Unreleased]

## [0.1.3] - 2026-06-30

### Changed

- Repositioned the project for discoverability: cargo-target-gc complements
  scoped `cargo clean` by adding age/category-based cleanup, scan-first
  reporting, and warm-cache preservation by default.
- Improved crates.io search metadata with cleanup/cache-oriented keywords.
- Added README discoverability sections comparing cargo-target-gc with scoped
  `cargo clean`, showing anonymized reclaimable-space examples, and documenting
  faster install options.
- Added maintainer-facing announcement and curated-list outreach materials.
- Added cargo-dist release automation for GitHub release archives and
  shell/PowerShell installers.

## [0.1.2] - 2026-06-30

### Added

- `clean --profile-cache` for explicitly reclaiming fresh incremental cache and
  recent Cargo profile cache directories such as `deps`, `build`,
  `.fingerprint`, and `examples`.
- Scan output now reports profile cache bytes separately from ordinary retained
  artifacts.
- Documentation explaining how cargo-target-gc differs from both plain and
  scoped `cargo clean` when users want to preserve some build-cache value.

## [0.1.1] - 2026-06-30

### Added

- `install-agent-skills` command for installing cargo-target-gc skills into
  Claude Code and Codex host-agent skill directories.
- Agent skill guidance that scans first, prefers dry runs, and requires explicit
  user approval before confirmed cleanup.
- README and localized quick-start motivation for vibe coding and agentic coding
  workflows that grow Cargo `target/` directories quickly.

## [0.1.0] - 2026-06-30

### Added

- Cargo `target/` scan reports for reclaimable and retained artifacts.
- Confirmed clean mode guarded by dry-run/confirm semantics.
- Active Cargo/rustc process detection before confirmed clean.
- Fresh incremental cache retention for edit-build performance.
- Documentation, license files, contribution guide, security policy, and GitHub
  project templates for open source readiness.
- Localized quick-start documentation in Arabic, Chinese, Dutch, Finnish,
  French, German, Hindi, Indonesian, Japanese, Korean, Portuguese, Russian,
  Spanish, Swahili, Swedish, and Vietnamese.

### Changed

- Project positioning is now focused on Cargo target artifact garbage
  collection.

### Safety

- `scan` remains read-only and never invokes Cargo.
- `clean` removes only analyzed reclaimable artifacts inside validated target
  roots.
