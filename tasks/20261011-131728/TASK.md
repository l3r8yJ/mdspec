# Provide a pre-commit hook

- STATUS: OPEN
- PRIORITY: 85
- TAGS: enhancement,v0.2.0

## Problem

There is no ready hook for running mdspec on commit.

## Proposal

Add `.pre-commit-hooks.yaml` with an `mdspec` hook (`language: rust`,
`files: \.md$`, `entry: mdspec check`).

## Acceptance

- `pre-commit try-repo` runs the hook on a sample repo and reports
  violations.
