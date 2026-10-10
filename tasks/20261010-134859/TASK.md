# Support multiple endpoints in one document

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: enhancement, v0.1.1
- GITHUB: #10

## Problem

`mdspec` assumes one endpoint per document: MDS001 expects exactly one `## <endpoint_prefix> name` heading, and section order, heading levels and the `References` placement are all checked against that single endpoint.

Real service docs keep every endpoint of a service in one `readme.md`. A service README with ~120 endpoints fails with 464 errors, none of which point at a real defect:

| Rule | Count | Cause |
| --- | --- | --- |
| MDS004 | 178 | `Logic`/`Components`/`Mappings` repeat once per endpoint and are seen as duplicates |
| MDS015 | 163 | there is one shared `References` block at the end of the file, not one per endpoint |
| MDS001 | 119 | every endpoint after the first is reported as an extra H2 |
| MDS005 | 1 | the document starts with an `# Service name` H1 |
| MDS002 | 1 | a preamble H2 (`## General rules`) precedes the first endpoint |

Cutting the same file into one document per endpoint (each with the shared reference definitions appended) gives 116 files and 0 errors, so the per-endpoint content is valid — only the document-level model is too narrow.

## Proposal

Treat each `## <endpoint_prefix> …` heading as the start of an endpoint block that runs to the next H2:

- MDS001: require at least one endpoint heading (when `require_endpoint = true`) instead of exactly one.
- MDS002, MDS003, MDS004, MDS007, MDS008, MDS009, MDS010: evaluate per endpoint block. Section order and uniqueness reset at each endpoint; component names are unique within their endpoint.
- Allow an optional H1 title and non-endpoint H2 sections (preamble such as general rules) before, between or after endpoint blocks; they are not checked against the endpoint section rules.
- MDS015: allow a single document-level references section at the end of the file (e.g. `## References`) that serves all endpoints, in addition to the current per-endpoint `### References`.
- Anchors (MDS013) and reference definitions (MDS011/MDS016) stay document-wide, so endpoints can link to each other's components.

Opt-in keeps current behavior unchanged, e.g.:

```toml
[document]
multiple_endpoints = true
```

## Acceptance

- A fixture with an H1, a preamble H2, three endpoint blocks and one trailing references section passes.
- The same fixture with sections out of order in the second endpoint reports MDS004 on that endpoint's line only.
- Duplicate component names across different endpoints pass; within one endpoint they still fail MDS009.
- With `multiple_endpoints = false`, existing fixtures and diagnostics are unchanged.
