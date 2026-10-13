# bump version on rultor release

- STATUS: OPEN
- PRIORITY: 110
- TAGS: ci,v0.2.0

Current implementation of release script requires a manual bump of version in `toml` configuration.
We need to change this behavior to rultor bump this version automatically by committing it directly before release.
Value of the release-tag version have to be taken from release message.
