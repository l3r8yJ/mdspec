# Replace rule ID string literals with a Rule enum

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: refactor,v0.1.4

## Problem

Rule IDs are string literals across `src/`, and `reporter.rs` picks exit
code 2 by matching `"MDS900" | "MDS901" | "MDS902"`. A typo or a new
operational rule fails silently.

## Proposal

A `Rule` enum with one variant per rule, serialized to the same strings,
and a method telling operational rules apart. JSON stays `schema_version`
1 with the same `rule` values.

## Acceptance

- No `"MDS` string literal remains in `src/` outside the enum.
- JSON output for all fixtures is byte-identical before and after.
- Exit code 2 comes from the enum, not string matching.
