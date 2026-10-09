#!/usr/bin/env bash
set -euo pipefail
: "${RELEASE_TAG:?Rultor release tag is required}"
just verify-tag
commit_sha=$(git rev-parse HEAD)
ci_run=$(curl --fail --silent --show-error --connect-timeout 10 --max-time 30 --retry 2 \
    --get --data-urlencode "head_sha=$commit_sha" --data-urlencode branch=main \
    --data-urlencode event=push --data-urlencode per_page=1 \
    https://api.github.com/repos/l3r8yJ/mdspec/actions/workflows/ci.yml/runs)
if ! jq --exit-status --arg sha "$commit_sha" \
    '.workflow_runs[0] | .head_sha == $sha and .head_branch == "main" and .event == "push" and .status == "completed" and .conclusion == "success"' \
    <<< "$ci_run" > /dev/null; then
    echo 'Release requires successful main push CI for this exact commit.' >&2
    exit 1
fi
cargo_directory="${CARGO_HOME:-$HOME/.cargo}"
credential_path="$cargo_directory/credentials.toml"
if [[ -e "$credential_path" || -e "$cargo_directory/credentials" ]]; then
    echo 'Refusing to overwrite existing Cargo credentials.' >&2
    exit 1
fi
test -s ../credentials
mkdir -p "$cargo_directory"
trap 'rm -f -- "$credential_path"' EXIT
install -m 600 ../credentials "$credential_path"
just publish
