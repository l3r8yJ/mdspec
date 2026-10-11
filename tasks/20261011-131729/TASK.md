# Provide a GitHub Action

- STATUS: OPEN
- PRIORITY: 85
- TAGS: enhancement,v0.2.0

## Problem

GitHub users have to download and unpack a release archive themselves.

## Proposal

A composite action in this repo that downloads the musl archive for a
given version, checks it against `SHA256SUMS`, and runs
`mdspec check` with `--format github`.

## Acceptance

- A workflow using the action annotates a PR with a violation.
- A checksum mismatch fails the action.
