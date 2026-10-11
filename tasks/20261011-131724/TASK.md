# Report duplicate endpoint paths

- STATUS: OPEN
- PRIORITY: 55
- TAGS: enhancement,v0.2.0

## Problem

Two endpoints with the same `Path: GET /a` in one document pass. In a
README with ~120 endpoints a copy-paste duplicate is easy to miss.

## Proposal

New rule MDS018 on the second and later endpoints whose method and path
match an earlier endpoint in the same document.

## Acceptance

- Two endpoints with the same method and path report MDS018 on the second.
- Same path with different methods passes.
- README documents MDS018.
