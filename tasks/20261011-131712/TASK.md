# Name the invalid label in the MDS900 label message

- STATUS: OPEN
- PRIORITY: 30
- TAGS: bug,v0.1.4

## Problem

`Labels::is_valid` rejects empty, padded or multi-line labels, duplicate
section labels, `source == target` and `request == response`, but the
message only mentions source and target and never names the offending key.
A config with `request = "X"` and `response = "X"` exits 2 with a message
that does not apply.

## Proposal

Return which `labels.<key>` failed and why, e.g.
`labels.request and labels.response must differ`. Report every invalid key.

## Acceptance

- Equal request and response labels report MDS900 naming both keys.
- A label with surrounding whitespace reports MDS900 naming that key.
- Duplicate section labels report MDS900 naming both keys.
