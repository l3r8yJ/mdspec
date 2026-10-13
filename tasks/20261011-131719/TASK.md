# Fail CI on code duplication

- STATUS: CLOSED
- PRIORITY: 55
- TAGS: ci,v0.1.4

## Problem

Nothing stops copy-pasted code: clippy has no general duplication lint.
jscpd 4.3.0 with `--min-tokens 50` finds 21 clones, 183 lines (4.95%), in
`src` and `tests`; `src` alone is 0.54%.

## Proposal

Add a `just duplication` recipe and CI job running jscpd on `src` and
`tests` with a pinned version, `--format rust`, `--min-tokens 50` and
`--threshold` set just above the level reached after the shared test
helper task. Lower the threshold as duplication drops.

## Acceptance

- CI fails when duplication exceeds the threshold.
- The job uses a pinned jscpd version.
- The job becomes a required check on `main`.
