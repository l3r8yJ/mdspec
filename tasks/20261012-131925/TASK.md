# Fail closed when the CI change detection fails

- STATUS: OPEN
- PRIORITY: 60
- TAGS: ci,v0.1.4

## Problem

`ci.yml` and `mutation.yml` gate their jobs on a `changes` job that reports
whether anything outside `tasks/` changed:

```yaml
format:
  needs: changes
  if: needs.changes.outputs.code == 'true'
```

If the `changes` job itself fails — checkout failure, runner problem, or an
error before the filter step — then `needs.changes.outputs.code` is empty, the
condition is false, and every gated job is skipped. `clippy`, `test` and
`build` cascade-skip through their existing `needs:` chains, and the mutation
job skips the same way.

A skipped job reports as a passing check for branch protection. That is
verified, not assumed: a tasks-only pull request against `main` comes out
mergeable with all five required checks showing `SKIPPED`. So a failure in the
gate turns the entire required-check set green and the pull request can merge
with no CI at all.

The filter step already fails safe internally — an empty, all-zero or
unresolvable base all resolve to `code=true`. The gap is the job failing
before or outside that step, and the `needs` condition treating "no result"
the same as "only tasks changed".

## Proposal

Run the job unless the filter positively reported tasks-only:

```yaml
if: always() && (needs.changes.result != 'success' || needs.changes.outputs.code == 'true')
```

Apply to `format`, `dependencies` and `version` in `ci.yml`, and to `mutation`
in `mutation.yml`. The cascade jobs need no change.

## Acceptance

- A `changes` job that fails leaves the required checks running or failing,
  never skipped and green.
- A tasks-only pull request still skips every gated job.
- A normal pull request still runs the full set.
