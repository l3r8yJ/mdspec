# Strip a leading UTF-8 BOM before parsing a document

- STATUS: OPEN
- PRIORITY: 10
- TAGS: bug,v0.1.4

## Problem

A file saved with a UTF-8 BOM (`EF BB BF`), as many Windows editors do, fails
even when valid: `tests/fixtures/minimal.md` with a BOM reports MDS001 and
MDS005 at 5:1 and exits 1. Columns on line 1 are also shifted by one. The
same happens for anchor targets read in `references::target_anchors`, so a
link to the first heading of a BOM-prefixed file reports MDS013.

## Proposal

Strip a leading U+FEFF in `parser::parse`, which covers both the checked
document and link targets. Positions keep the original byte offsets.
TOML config already accepts a BOM and stays unchanged.

## Acceptance

- `minimal.md` with a BOM passes with no diagnostics.
- A link at column 6 on line 1 of a BOM file is reported at column 6.
- An anchor link to the first heading of a BOM target file resolves.
