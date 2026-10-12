# Add quality gates to CI and repository settings

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: enhancement,v0.1.4

Add automated gates so regressions fail CI instead of being found in review:

- `cargo deny check`: advisories, yanked crates, licenses, duplicate
  versions, wildcard requirements, unknown sources (`deny.toml`).
- `cargo machete`: unused dependencies.
- `just verify-version`: `Cargo.toml` is not behind the latest `v*` tag.
- `tests/readme.rs`: the README quick start and examples pass mdspec.
- Repository settings: squash merges only, delete merged branches, protect
  `main` with the CI checks required.

Then record what the gates catch as follow-up tasks.
