# Validate JSON request and response examples

- STATUS: OPEN
- PRIORITY: 60
- TAGS: enhancement,v0.2.0

## Problem

A fenced `json` block under a request or response example is never parsed,
so invalid JSON passes.

## Proposal

Parse example code blocks whose info string is `json` with `serde_json`
(already a dependency) and report MDS017 with the parse error. Other
languages are not checked.

## Acceptance

- Invalid JSON in a `json` example reports MDS017 with line information.
- Valid JSON and non-JSON blocks keep passing.
