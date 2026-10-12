# Clean stale coverage profiles before measuring coverage

- STATUS: CLOSED
- PRIORITY: 55
- TAGS: ci,v0.1.4

## Problem

`just coverage` fails after any Rust toolchain change, with an error that names
LLVM versions rather than the leftover files causing it:

```
raw profile version mismatch: Profile uses raw profile format version = 11; expected version = 10
error: no profile can be merged
error: recipe `coverage-check` failed on line 33 with exit code 1
```

`just test-coverage` runs `cargo llvm-cov --no-report`, which is deliberately
designed to accumulate raw profiles across several runs so a later `report`
can merge them. Nothing ever cleans `target/llvm-cov-target/`, so profiles
written under one toolchain stay behind and the current toolchain's
`llvm-profdata` refuses to merge them.

This was hit for real on 2026-10-12: Renovate bumped `rust-toolchain.toml` to
1.99.0, the bump was reverted to 1.96.0, and 2232 accumulated `.profraw` files
from both eras made `just coverage` — and therefore `just full` — fail. It does
not affect CI, where each run starts from a clean checkout, so the gate that
protects the repository never sees it. It only breaks the local `just full`
that contributors are told to run.

## Proposal

Clean the profraw directory before the instrumented run, so accumulated state
from another toolchain cannot poison a measurement:

```
test-coverage:
    cargo llvm-cov clean --workspace
    cargo llvm-cov --locked --all-features --no-report
```

Confirm `clean --workspace` removes what it needs to without discarding the
build cache in a way that makes the recipe materially slower; if it does, scope
the cleanup to the profile data instead.

## Acceptance

- `just coverage` passes after switching `rust-toolchain.toml` to a different
  version and back, without a manual cleanup step.
- `just full` passes on a checkout whose `target/llvm-cov-target/` holds
  profiles from a different toolchain.
- The coverage number for an unchanged tree is the same before and after.
