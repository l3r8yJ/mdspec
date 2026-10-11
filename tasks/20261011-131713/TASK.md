# Share the test binary helper in tests/common

- STATUS: OPEN
- PRIORITY: 40
- TAGS: refactor,v0.1.4

## Problem

Most files in `tests/` define their own helper that writes files to a temp
dir, runs `CARGO_BIN_EXE_mdspec` and parses the JSON report. jscpd finds 21
clones (183 lines, 4.95%) across `src` and `tests`; all but one are in
`tests/`, mostly `selection.rs`, `cli.rs` and `output_and_discovery.rs`.

## Proposal

Move the helpers into `tests/common/mod.rs` (`mod common;` in each test
file): run the binary with a document and config, return rules with lines.

## Acceptance

- No test file defines its own binary-running helper.
- Test names and assertions are unchanged; the suite still passes.
- jscpd duplication in `tests/` drops below 2%.
