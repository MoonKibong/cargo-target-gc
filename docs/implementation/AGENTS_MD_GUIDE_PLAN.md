---
order: agents-md-guide
updated: 2026-06-29T06:29:53Z
---

# Plan: agents-md-guide

---
order: agents-md-guide
updated: 2026-06-29T06:28:34Z
---

# Plan: agents-md-guide — Repository Contributor Guide (AGENTS.md)

## Context & Decisive Finding
AGENTS.md ALREADY EXISTS at repo root (`/Users/kibong/development/derust/AGENTS.md`, 2.7K, 42 lines), titled "Repository Guidelines". The order's create action is GUARDED: "if it exists, do not overwrite or modify it." The guard condition is met, so the executor must NOT create, overwrite, or edit AGENTS.md. The existing file already covers all six required topics: Project Structure & Module Organization; Build/Test/Dev Commands; Coding Style & Naming; Testing Guidelines; Commit & Pull Request Guidelines; and Agent-Specific Instructions.

This plan is therefore a VERIFICATION plan that confirms the guard is honored and the existing guide is adequate — no file mutation occurs.

`docs/dev/pipeline-hardening-notes.md` is absent; no prior failure modes to reference.
No quality gates triggered: no production file >1000 lines, no code duplication introduced (no code changes at all).

## Affected repositories & dependency order
Single repo: `/Users/kibong/development/derust`. Tasks run sequentially (T1 -> T2 -> T3); each depends on the prior.

## Task T1 — Confirm guard condition (AGENTS.md exists -> do not modify)
Capture a baseline checksum of `AGENTS.md` so any later mutation is detectable.
- Action: run `test -f AGENTS.md && shasum -a 256 AGENTS.md` from repo root; record the digest.
- Depends on: none.
- Acceptance: `test -f AGENTS.md` exits 0 and `shasum -a 256 AGENTS.md` prints a 64-hex-char SHA-256 digest that is recorded verbatim in the executor report for reuse in T3. No Write/Edit tool is invoked on `AGENTS.md` (verified by absence of edit tool calls in the task log).

## Task T2 — Verify content completeness against the order's required topics
Confirm the existing `AGENTS.md` satisfies all six required sections WITHOUT editing it.
- Action: run `grep -E '^## ' AGENTS.md` and confirm headings for: Project Structure & Module Organization; Build, Test, and Development Commands; Coding Style & Naming Conventions; Testing Guidelines; Commit & Pull Request Guidelines; Agent-Specific Instructions.
- Depends on: T1.
- Acceptance: `grep -cE '^## ' AGENTS.md` returns `6` and the six required headings above each appear in the `grep -E '^## ' AGENTS.md` output (6/6 match), documented in the executor report. No edits made. If any required section is missing, the executor MUST stop and escalate rather than edit the protected file.

## Task T3 — Final verification: prove AGENTS.md was left unmodified
Verification step: the Makefile build/test/lint targets are TODO stubs that only `echo`, so the real deliverable is the no-mutation invariant rather than a passing test suite.
- Action: re-run `shasum -a 256 AGENTS.md` and compare to the T1 baseline; run `git status --porcelain AGENTS.md`.
- Depends on: T2.
- Acceptance: the T3 `shasum -a 256 AGENTS.md` digest is byte-for-byte equal to the T1 digest recorded in T1's acceptance (string equality), AND `git status --porcelain AGENTS.md` shows the file as untracked (`??`) — i.e. not staged-modified — confirming this order changed nothing. Executor report ends with the line `guard honored; existing guide adequate; no changes made`.

## Risks
- R1 (primary): Executor mistakes the order's create-intent for a mandate and overwrites the existing `AGENTS.md`. Mitigation: T1/T3 checksum gate makes any mutation a hard verification failure.
- R2: Existing `AGENTS.md` states "repository is currently empty," which is now mildly stale (`docs/`, `Makefile`, `README.md` exist). The guard FORBIDS modifying `AGENTS.md`, so no update is planned. See escalation below.

## Escalation to PM (user decision needed)
The order is conditional and the condition resolves to "do nothing": `AGENTS.md` exists, so it must not be touched. Net effect of this order is verification only. Additionally, the existing `AGENTS.md`'s "repository is currently empty" line is slightly outdated. Two options for the user:
  (a) Accept as-is — honor the guard, make no changes (this plan's default).
  (b) Issue a NEW order explicitly authorizing an update to `AGENTS.md` to refresh the stale "empty repository" wording and align with the current `docs/` + `Makefile` layout.
Default to (a) unless the user authorizes (b).

