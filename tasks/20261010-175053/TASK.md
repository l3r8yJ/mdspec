# Publish a static x86_64-unknown-linux-musl release build

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: enhancement
- GITHUB: #18

## Problem

Releases ship only `mdspec-<version>-x86_64-unknown-linux-gnu.tar.gz`, a binary dynamically linked against glibc (`interpreter /lib64/ld-linux-x86-64.so.2`). It does not start on Alpine, which is the base of most CI lint images:

```
$ docker run --rm -v ./mdspec:/mdspec:ro --entrypoint /mdspec <alpine-based-image> --version
exec /mdspec: no such file or directory
```

The image only has `/lib/ld-musl-x86_64.so.1`. So mdspec cannot be added to an Alpine-based CI job or lint image from a release artifact; the only path is building from source with a Rust toolchain in the image.

## Proposal

Add an `x86_64-unknown-linux-musl` build to the release, statically linked, so it runs on Alpine and on any other x86_64 Linux without a libc dependency.

- `justfile` (`dist` recipe, currently hardcodes the gnu target at line 53) and `.github/workflows/release.yml` build `--target x86_64-unknown-linux-musl` in addition to the gnu target.
- Upload `mdspec-<version>-x86_64-unknown-linux-musl.tar.gz` next to the gnu archive, with the same layout (single `mdspec` at the archive root).
- `SHA256SUMS` lists both archives.
- README Install section mentions the musl build for Alpine.

## Acceptance

- The release has `mdspec-<version>-x86_64-unknown-linux-musl.tar.gz` and its line in `SHA256SUMS`.
- `file mdspec` on the extracted binary reports `static-pie linked` or `statically linked`.
- `docker run --rm -v "$PWD/mdspec:/mdspec:ro" alpine:3 /mdspec --version` prints the version; this check runs in the release workflow before upload.
