---
name: Unsafe cleanup report
about: Report suspected deletion outside cargo-target-gc's intended scope
title: "security: unsafe cleanup report"
labels: security
assignees: ""
---

Do not include private project details in a public issue. If the repository has
private vulnerability reporting enabled, use that channel instead.

## Summary

What was removed or planned for removal unexpectedly?

## Command

```bash
cargo target-gc clean ...
```

## Project Layout

Describe the relevant Cargo workspace, target directories, symlinks, and config.

## Expected Scope

What should cargo-target-gc have scanned or cleaned?

## Actual Scope

What did cargo-target-gc scan or clean?
