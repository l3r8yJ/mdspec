# Print text diagnostics to stdout

- STATUS: OPEN
- PRIORITY: 95
- TAGS: enhancement,v0.2.0

## Problem

Text output, diagnostics and summary, goes entirely to stderr; stdout is
empty, so `mdspec check docs | grep MDS011` matches nothing.

## Proposal

Diagnostics and the summary go to stdout; stderr is only for failures to
write the report. Breaking change, so it ships in a minor version.

## Acceptance

- `--format text` diagnostics and summary are on stdout, stderr is empty.
- `--format json` is unchanged.
- README and the changelog note the change.
