# Generate duplicate heading anchors with a counter

- STATUS: OPEN
- PRIORITY: 50
- TAGS: refactor,v0.1.4

## Problem

`anchors()` in `src/references.rs` deduplicates with
`while anchors.contains(..)`. The `+=` to `*=` mutant made it loop forever
and timed out the mutation job; tests now avoid three equal headings to
dodge it.

## Proposal

Count occurrences per base slug in a map, as GitHub and GitLab do, with no
unbounded loop. Keep the current results, including a heading literally
named `foo-1` after two `foo` headings.

## Acceptance

- Same anchors as today for all existing tests.
- A document with three equal headings resolves `#x`, `#x-1`, `#x-2`.
- The mutation job kills the counter mutants without a timeout.
