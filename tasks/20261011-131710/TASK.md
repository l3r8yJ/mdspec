# Report unused definitions as warnings without --strict

- STATUS: OPEN
- PRIORITY: 20
- TAGS: bug,v0.1.4

## Problem

MDS016 (unused reference definition) is the only warning, and it is only
produced under `--strict`, which also turns every warning into an error. So
`warning_count` is always 0 and `[diagnostics] warnings_as_errors = true`
has no observable effect. README never mentions `warnings_as_errors` and
says MDS016 is "Only with `--strict`".

## Proposal

Always report MDS016 as a warning (exit code stays 0). `--strict` or
`warnings_as_errors = true` turn warnings into errors (exit code 1).
Behavior change: JSON consumers that fail on any diagnostic will now see
MDS016 without `--strict`.

## Acceptance

- Without `--strict`: an unused definition gives one MDS016 warning,
  `warning_count` 1, exit 0.
- With `--strict` or `warnings_as_errors = true`: MDS016 is an error, exit 1.
- README documents `warnings_as_errors` and the new MDS016 behavior.
