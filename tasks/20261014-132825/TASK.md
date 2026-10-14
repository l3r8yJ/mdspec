# Always allow multiple endpoints

- STATUS: OPEN
- PRIORITY: 130
- TAGS: refactor,v0.2.0

`document.multiple_endpoints` forks validation into two code paths across `rules.rs`, `references.rs` and `components.rs`. Single-endpoint mode is a special case nobody needs: a document with one endpoint is already valid in multiple-endpoint mode.

- Remove the `document.multiple_endpoints` key. Every document may hold any number of `## <endpoint_prefix> name` endpoints.
- Delete the single-endpoint code paths and their tests.
- A config that still sets the key is rejected with MDS900, like any unknown key. This is a breaking change, called out in the release notes.
