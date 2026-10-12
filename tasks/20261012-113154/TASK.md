# Ignore the .claude directory

- STATUS: OPEN
- PRIORITY: 60
- TAGS: ci,v0.1.4

## Problem

`.claude/` is untracked but not ignored, so it appears in every `git status`
and is swept into anything that copies the working tree. `cargo-mutants`
copies the tree once per worker into `/tmp`, which is a 16G tmpfs; agent
worktrees left under `.claude/worktrees/<agent>/target/` added 6.4G to that
copy and `just full` died with:

```
Failed to copy .claude/worktrees/<agent>/target/debug/deps/endpoints-…
No space left on device (os error 28)
```

The top-level `/target/` entry already keeps cargo's own output out of that
copy; `.claude/` needs the same treatment.

## Proposal

Add `/.claude/` to `.gitignore` next to the existing `/target/`, `/dist/`
and `/mutants.out/` entries.

## Acceptance

- `git check-ignore -v .claude` reports the rule.
- `git status` stays clean when a tool leaves `.claude/` behind.
