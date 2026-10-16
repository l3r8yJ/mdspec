# Name the magic numbers

- STATUS: CLOSED
- PRIORITY: 112
- TAGS: refactor,v0.2.0

Meaningful values appear as bare numeric literals, so a reader has to know the heading scheme or the CLI contract to follow the code: `children(blocks, index, 3)`, `matches!(…, Heading { level: 4, .. })`, `ExitCode::from(2)`, `schema_version: 1`, `to_digit(16)`, `rows.len() < 2`.

- Heading levels get names by role (title, part, section, section item, item description; general-section item and its description), defined once and used everywhere a level is matched or passed.
- Exit codes are named after their meaning (clean, violations, operational error) in one place used by both `main.rs` and the reporter.
- The JSON schema version, the percent-encoding radix and the mapping table's minimum row count are named constants.
- Obvious index arithmetic (`0`, `+ 1`, `.skip(1)` over a header row) stays as it is.
- No diagnostic or exit-code change.
