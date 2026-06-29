# cargo-target-gc v0.1.3

Rust builds are great, but agentic coding can make `target/` directories grow
quickly. cargo-target-gc now has clearer discovery metadata, a sharper README,
and release-binary automation so Rust developers can find and install it more
easily.

## Why It Exists

`cargo clean` is useful and has scoped options such as `--package`, `--profile`,
`--release`, `--target`, `--target-dir`, and `--doc`. Those options still clean
whole selected scopes. cargo-target-gc complements Cargo by scanning first and
classifying artifacts by age and cache role, so users can reclaim disk space
while preserving more warm build cache by default.

## Highlights

- Updated crates.io keywords toward real search terms: `cargo`, `target`,
  `clean`, `cleanup`, `cache`.
- Added a README comparison of `cargo clean` and `cargo target-gc`.
- Added anonymized dry-run examples showing 52.7 GiB, 62.4 GiB, and 106.7 GiB
  reclaimable with `--profile-cache` from large local Rust projects.
- Added cargo-dist release automation for GitHub archives and shell/PowerShell
  installers.
- Added announcement copy and curated-list submission material under
  `docs/distribution/`.

## Install

```bash
cargo install cargo-target-gc --locked
```

With cargo-binstall:

```bash
cargo binstall cargo-target-gc
```

## First Run

```bash
cargo target-gc scan
cargo target-gc clean --dry-run
```

Use `clean --confirm` only after reviewing the dry run.
