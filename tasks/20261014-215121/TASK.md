# Decide what a late title means

- STATUS: CLOSED
- PRIORITY: 110
- TAGS: refactor,v0.2.0

An H1 that appears after the first endpoint (a "late title") is handled three different ways today, and the outline reproduces all three through `Endpoint.detached` and the builder's `Target::{Sections, Detached, Closed}` states, only to keep diagnostics unchanged during the outline refactor.

- Decide one meaning for a late H1 (for example: it is always MDS005 and nothing after it belongs to the previous endpoint, or it is simply ignored).
- Implement that meaning in `src/outline.rs` and delete `Endpoint.detached` and the extra builder states.
- Pin the chosen behavior with black-box tests and describe it in the README.
