# Discover mdspec.toml without --config

- STATUS: OPEN
- PRIORITY: 60
- TAGS: enhancement,v0.2.0

## Problem

Config is only read from `--config`, so a project's `mdspec.toml` is
ignored unless every caller passes the flag.

## Proposal

Without `--config`, use the first `mdspec.toml` found in the current
directory or its parents, stopping at the directory that contains `.git`.
`--config` overrides discovery; `--no-config` uses defaults. A discovered
file that fails to parse reports MDS900 with its path.

## Acceptance

- A `mdspec.toml` in the repo root applies when run from a subdirectory.
- `--config` wins over a discovered file; `--no-config` ignores it.
- No file found keeps the defaults.
