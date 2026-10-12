# Automate dependency and action updates

- STATUS: CLOSED
- PRIORITY: 60
- TAGS: ci,v0.1.4

## Problem

Nothing proposes updates for Cargo dependencies or the SHA-pinned GitHub
Actions. An old `renovate/configure` branch exists but was never merged.

## Proposal

Pick one of Renovate or Dependabot. Group Cargo minor and patch updates,
keep actions pinned by SHA with the version in a comment, weekly schedule.
Delete the stale `renovate/configure` branch if Dependabot is chosen.

## Acceptance

- Update PRs open for Cargo and Actions and run the full CI.
- Action pins stay full commit SHAs.
