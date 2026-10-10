# Support request and response examples for an endpoint

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: enhancement

## Problem

An endpoint document describes logic, components and mappings, but has no
validated place for a sample request and response. Authors either skip
examples or put them under an unknown section, which mdspec does not check
and rejects when `allow_unknown_sections = false`.

## Proposal

Add an optional examples section to the endpoint structure:

~~~md
### Примеры

#### Запрос

```json
{"name": "backup"}
```

#### Ответ

```json
{"id": 1, "name": "backup"}
```
~~~

- New labels: `examples` (default `Примеры`), `request` (default `Запрос`),
  `response` (default `Ответ`).
- The examples section sits after Mappings and before References in the
  enforced section order.
- Under it, H4 headings may only be the request or response label, each at
  most once per endpoint, each followed by a nonempty fenced code block.
- `document.require_examples = false` by default; when `true`, MDS003 reports
  a missing examples section.
- Violations get a new rule, MDS017.
- Works per endpoint when `multiple_endpoints = true`.

## Acceptance

- A document with a request and a response example passes.
- An examples section with an unknown H4, a duplicate request, or an H4
  without a fenced code block reports MDS017 on that heading.
- An examples section placed after References reports MDS004.
- Documents without an examples section keep their current diagnostics.
- README documents the labels, the config key and MDS017.
