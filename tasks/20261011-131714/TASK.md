# Name the mutation test files after the behavior they check

- STATUS: OPEN
- PRIORITY: 45
- TAGS: refactor,v0.1.4

## Problem

`tests/rules_mutations.rs` and `tests/references_mutations.rs` are named
after the tool that motivated them, not what they test.

## Proposal

First check what each test covers. Move rule tests into `structure.rs` or
`endpoints.rs` and reference tests into `references.rs`, then delete the
two files.

## Acceptance

- No test file name refers to mutation testing.
- Same test count before and after; mutation run on `main` still has
  0 missed.
