# Install a prebuilt just in the Rultor image

- STATUS: OPEN
- PRIORITY: 60
- TAGS: ci,v0.1.4

## Problem

`.rultor.yml` runs `cargo install just --version 1.58.0 --locked`, which
takes about 3 of the 5 minutes of every release.

## Proposal

Download the just 1.58.0 `x86_64-unknown-linux-musl` release archive from
github.com/casey/just, check its SHA-256 against a pinned value, and put
it on PATH.

## Acceptance

- Rultor release log shows no `Compiling` lines for just.
- A wrong checksum fails the install step.
