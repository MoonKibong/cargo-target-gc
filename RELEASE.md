# Release Guide

This guide is for maintainers publishing cargo-target-gc.

## Release Policy

Best call: release the first public version as `0.1.0` and describe it as a
safe beta. cargo-target-gc deletes files when confirmed, so avoid production-grade claims
until real-world workspace feedback confirms the safety model.

Use semantic versioning:

- Patch: documentation fixes, bug fixes, narrower safety checks.
- Minor: new flags, report fields, config keys, or supported project layouts.
- Major: breaking CLI, JSON, config, or cleanup semantics.

## Pre-Release Checklist

1. Confirm the README describes cargo-target-gc as a Cargo `target/` garbage collector.
2. Confirm `CHANGELOG.md` has release notes for the version.
3. Confirm `Cargo.toml` has the real public `repository` URL before crates.io.
4. Run:

```bash
make fmt-check
make lint
make test
cargo package --locked
cargo publish --dry-run
```

5. Manually test:

```bash
cargo target-gc scan
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
```

Use a disposable project or a known fixture before any confirmed clean.

## Publishing

1. Update `version` in `Cargo.toml`.
2. Move `CHANGELOG.md` entries from `Unreleased` to the release version.
3. Commit the release changes.
4. Tag the commit, for example:

```bash
git tag v0.1.0
```

5. Publish:

```bash
cargo publish
```

6. Push the branch and tag, then create a GitHub release from the changelog.

Pushing a version tag also triggers the generated cargo-dist workflow in
`.github/workflows/release.yml`. That workflow builds release archives and shell
/ PowerShell installers for GitHub Releases. If the workflow fails, keep the
crates.io release intact and fix the binary-release workflow in a patch release.

## Rollback

If a release has a cleanup-safety bug, yank the crates.io version and publish a
security note:

```bash
cargo yank cargo-target-gc --version <version>
```

Do not delete the Git tag; create a follow-up patch release with the fix.
