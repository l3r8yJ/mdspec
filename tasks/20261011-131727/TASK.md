# Publish a container image

- STATUS: OPEN
- PRIORITY: 80
- TAGS: enhancement,v0.2.0

## Problem

GitLab CI users need a Rust toolchain or a download step to run mdspec.

## Proposal

Publish `ghcr.io/l3r8yj/mdspec:<version>` and `:latest`, built `FROM
scratch` with the musl binary as entrypoint, from the release workflow.

## Acceptance

- `docker run ghcr.io/l3r8yj/mdspec:<version> --version` prints the
  version.
- The image is pushed only after the release archives are verified.
