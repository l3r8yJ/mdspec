# Publish an aarch64-unknown-linux-musl build

- STATUS: OPEN
- PRIORITY: 80
- TAGS: enhancement,v0.2.0

## Problem

Release archives exist only for x86_64, so ARM CI runners and hosts need
to build from source.

## Proposal

Build `aarch64-unknown-linux-musl` on a `ubuntu-24.04-arm` runner with the
same smoke test (`file` reports static, `--version` runs in `alpine:3`),
and attach the archive to the release with its line in `SHA256SUMS`.

## Acceptance

- The release has an `aarch64-unknown-linux-musl` archive and checksum.
- The arm binary passes the smoke test in CI before upload.
