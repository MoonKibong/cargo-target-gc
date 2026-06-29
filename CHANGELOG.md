# Changelog

All notable changes to cargo-target-gc will be documented in this file.

The format follows Keep a Changelog, and this project uses semantic versioning
once public releases begin.

## [Unreleased]

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
