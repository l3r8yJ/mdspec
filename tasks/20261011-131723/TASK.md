# Add GitHub and GitLab report formats

- STATUS: OPEN
- PRIORITY: 70
- TAGS: enhancement,v0.2.0

## Problem

CI integration needs JSON post-processing; neither GitHub annotations nor
GitLab merge request widgets are supported.

## Proposal

- `--format github`: one workflow command per diagnostic,
  `::error file=<path>,line=<l>,col=<c>,title=<rule>::<message>`
  (`::warning` for warnings), positions omitted when unknown.
- `--format gitlab`: Code Quality JSON array with `description`,
  `check_name`, `fingerprint` (stable hash of rule, path, line, message),
  `severity` and `location.path` / `location.lines.begin`.
- SARIF later, when there is demand.

## Acceptance

- Both formats are covered by tests against the documented field names.
- Exit codes are the same as for the other formats.
