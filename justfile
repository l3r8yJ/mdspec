set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

fmt:
    cargo fmt

fmt-check:
    just --fmt --check
    cargo fmt --check

clippy:
    cargo clippy --locked --all-targets --all-features -- -D warnings

test:
    cargo test --locked --all-features

[positional-arguments]
mutate *args:
    cargo mutants --jobs 2 --build-timeout 300 "$@"

test-coverage:
    cargo llvm-cov --locked --all-features --no-report

coverage-check:
    cargo llvm-cov report --summary-only --fail-under-lines 70
    mkdir -p target/coverage
    cargo llvm-cov report --lcov --output-path target/coverage/lcov.info

coverage: test-coverage coverage-check

build:
    cargo build --locked --release

publish-check:
    cargo publish --locked --dry-run

publish:
    cargo publish --locked --no-verify

smoke: build
    ./target/release/mdspec check tests/fixtures/languages/english.md --config tests/fixtures/languages/english.toml --strict

package: smoke
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "mdspec") | .version')
    mkdir -p dist
    tar -czf "dist/mdspec-${version}-x86_64-unknown-linux-gnu.tar.gz" -C target/release mdspec
    cd dist
    sha256sum ./*.tar.gz > SHA256SUMS
    sha256sum --check SHA256SUMS

verify-tag:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "mdspec") | .version')
    if [[ "$RELEASE_TAG" != "v${version}" ]]; then
        echo "Release tag ${RELEASE_TAG} must match Cargo.toml version v${version}" >&2
        exit 1
    fi

ci: fmt-check clippy coverage package publish-check
