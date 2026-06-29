# Discoverability Plan

This is the maintainer checklist for making cargo-target-gc findable beyond a
crates.io publish.

## Current Positioning

cargo-target-gc is a Cargo target-artifact garbage collector for Rust developers
whose `target/` directories grow quickly during agentic coding sessions. It is
not a replacement for `cargo clean`; it is an age/category-based cleanup tool for
users who want a dry-run-first workflow and more control over build cache value.

## Search Surfaces

- crates.io keywords: `cargo`, `target`, `clean`, `cleanup`, `cache`
- GitHub topics: `rust`, `cargo`, `cargo-subcommand`, `target-directory`,
  `cleanup`, `disk-space`, `build-cache`, `developer-tools`
- README search phrases:
  - cargo clean alternative
  - clean Cargo target directory
  - Rust target directory disk usage
  - agentic coding build cache cleanup
  - cargo target cache cleanup

## Launch Checklist

1. Keep `Cargo.toml` metadata current before every crates.io release.
2. Create a GitHub release with the practical problem statement and dry-run
   evidence.
3. Post the announcement copy from `ANNOUNCEMENTS.md` to Rust communities.
4. Watch for feedback about false positives, confusing docs, and missing target
   layouts before broadening cleanup behavior.
5. After the project has early user validation, submit it to curated Rust tool
   lists using `CURATED_LISTS.md`.

## Install Friction

The primary install path is:

```bash
cargo install cargo-target-gc --locked
```

Also document:

```bash
cargo binstall cargo-target-gc
```

When release binaries are available, `cargo-binstall` can avoid a local compile.
Use cargo-dist or equivalent generated release automation for binaries; do not
hand-write release archive logic.
