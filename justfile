set shell := ["bash", "-euo", "pipefail", "-c"]

targets := "x86_64-unknown-linux-gnu x86_64-unknown-linux-musl"
static_target := "x86_64-unknown-linux-musl"

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
    #!/usr/bin/env bash
    set -euo pipefail
    workers="${CARGO_MUTANTS_JOBS:-$(( ({{ num_cpus() }} + 3) / 4 ))}"
    cargo mutants --jobs "$workers" --build-timeout 300 "$@"

test-coverage:
    cargo llvm-cov --locked --all-features --no-report

coverage-check:
    cargo llvm-cov report --summary-only --fail-under-lines 90
    mkdir -p target/coverage
    cargo llvm-cov report --lcov --output-path target/coverage/lcov.info

coverage: test-coverage coverage-check

build:
    for target in {{ targets }}; do cargo build --locked --release --target "$target"; done

publish-check:
    cargo publish --locked --dry-run

publish:
    cargo publish --locked --no-verify

smoke: build
    for target in {{ targets }}; do "target/$target/release/mdspec" check tests/fixtures/languages/english.md --config tests/fixtures/languages/english.toml --strict; done
    file target/{{ static_target }}/release/mdspec | grep -E 'static-pie linked|statically linked'
    docker run --rm -v "$PWD/target/{{ static_target }}/release/mdspec:/mdspec:ro" alpine:3 /mdspec --version

package: smoke
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "mdspec") | .version')
    mkdir -p dist
    for target in {{ targets }}; do tar -czf "dist/mdspec-${version}-${target}.tar.gz" -C "target/${target}/release" mdspec; done
    cd dist
    sha256sum ./*.tar.gz > SHA256SUMS
    sha256sum --check SHA256SUMS

set-version version:
    sed -i '0,/^version = ".*"$/s//version = "{{ version }}"/' Cargo.toml
    sed -i '/^name = "mdspec"$/{n;s/^version = ".*"$/version = "{{ version }}"/}' Cargo.lock

verify-tag:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "mdspec") | .version')
    if [[ "$RELEASE_TAG" != "v${version}" ]]; then
        echo "Release tag ${RELEASE_TAG} must match Cargo.toml version v${version}" >&2
        exit 1
    fi

ci: fmt-check clippy coverage package publish-check

full: ci mutate
