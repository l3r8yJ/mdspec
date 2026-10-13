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
    cargo llvm-cov clean --profraw-only
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

deps:
    cargo deny --locked check
    cargo machete

duplication:
    npx --yes jscpd@4.3.0 src tests --format rust --min-tokens 50 --threshold 1.5 --noTips

release-bump:
    #!/usr/bin/env bash
    set -euo pipefail
    release_tag="${RELEASE_TAG:?RELEASE_TAG must hold the release tag, for example v1.2.3}"
    [[ "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "Release tag ${release_tag} must look like v1.2.3" >&2; exit 1; }
    version="${release_tag#v}"
    latest=$(git tag --list 'v*' --sort=-v:refname | head -n 1)
    latest="${latest#v}"
    if [[ -n "$latest" && "$(printf '%s\n%s\n' "$latest" "$version" | sort -V | tail -n 1)" != "$version" ]]; then
        echo "Release tag ${release_tag} is not newer than the latest release v${latest}" >&2
        exit 1
    fi
    current=$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "mdspec") | .version')
    if [[ "$current" == "$version" ]]; then
        echo "Cargo.toml already holds ${version}, nothing to bump"
        exit 0
    fi
    just set-version "$version"
    git add Cargo.toml Cargo.lock
    git -c user.name=rultor -c user.email=rultor@localhost commit --message "chore: bump version to ${release_tag}"

verify-tag:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "mdspec") | .version')
    if [[ "$RELEASE_TAG" != "v${version}" ]]; then
        echo "Release tag ${RELEASE_TAG} must match Cargo.toml version v${version}" >&2
        exit 1
    fi

ci: fmt-check clippy deps duplication coverage package publish-check

full: ci mutate
