# Link each diagnostic to its rule documentation

- STATUS: OPEN
- PRIORITY: 90
- TAGS: enhancement,v0.2.0

## Problem

`Diagnostic.help` is always null in the JSON report: it is declared and
serialized but never set.

## Proposal

Set `help` to the README section of the rule, e.g.
`https://github.com/l3r8yJ/mdspec#mds001-endpoint-heading`. If that is not
wanted, delete the field and bump `schema_version` instead.

## Acceptance

- Every diagnostic has a `help` URL whose anchor exists in README.
- Or `help` is gone and `schema_version` is 2.
