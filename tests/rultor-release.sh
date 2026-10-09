#!/usr/bin/env bash
set -euo pipefail
project_dir=$(cd "$(dirname "$0")/.." && pwd)
workspace=$(mktemp -d)
trap 'rm -rf -- "$workspace"' EXIT
mkdir -p "$workspace/bin" "$workspace/repo" "$workspace/cargo"
printf 'dummy-test-credential\n' > "$workspace/credentials"
cat > "$workspace/bin/git" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == 'rev-parse HEAD' ]]
printf 'abc123\n'
MOCK
cat > "$workspace/bin/curl" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
printf '{"workflow_runs":[{"head_sha":"%s","head_branch":"%s","event":"%s","status":"completed","conclusion":"%s"}]}\n' "${TEST_SHA:-abc123}" "${TEST_BRANCH:-main}" "${TEST_EVENT:-push}" "${TEST_CONCLUSION:-success}"
MOCK
cat > "$workspace/bin/just" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    verify-tag) [[ "$RELEASE_TAG" == v0.1.0 ]] ;;
    publish)
        cmp ../credentials "$CARGO_HOME/credentials.toml"
        [[ $(stat -c %a "$CARGO_HOME/credentials.toml") == 600 ]]
        touch ../published
        exit "${TEST_PUBLISH_EXIT:-0}"
        ;;
    *) exit 2 ;;
esac
MOCK
chmod +x "$workspace/bin/"*
run_release() {
    (cd "$workspace/repo" && env PATH="$workspace/bin:$PATH" CARGO_HOME="$workspace/cargo" RELEASE_TAG=v0.1.0 "$@" bash "$project_dir/scripts/rultor-release.sh")
}
assert_absent() {
    if [[ -e "$1" ]]; then
        printf 'Expected absent file: %s\n' "$1" >&2
        exit 1
    fi
}
run_release
[[ -f "$workspace/published" ]] || { echo 'Publication was not invoked' >&2; exit 1; }
assert_absent "$workspace/cargo/credentials.toml"
rm "$workspace/published"
for invalid in TEST_SHA=wrong TEST_BRANCH=feature TEST_EVENT=pull_request TEST_CONCLUSION=failure RELEASE_TAG=v9.9.9; do
    if run_release "$invalid"; then
        printf 'Invalid release accepted: %s\n' "$invalid" >&2
        exit 1
    fi
    assert_absent "$workspace/published"
    assert_absent "$workspace/cargo/credentials.toml"
done
if run_release TEST_PUBLISH_EXIT=1; then
    echo 'Publish failure was ignored' >&2
    exit 1
fi
[[ -f "$workspace/published" ]]
assert_absent "$workspace/cargo/credentials.toml"
rm "$workspace/published"
printf 'existing-credential\n' > "$workspace/cargo/credentials.toml"
if run_release; then
    echo 'Existing credential file was overwritten' >&2
    exit 1
fi
[[ $(cat "$workspace/cargo/credentials.toml") == existing-credential ]]
assert_absent "$workspace/published"
printf 'Rultor release checks passed\n'
