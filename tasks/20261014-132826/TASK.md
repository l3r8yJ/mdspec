# Clean up the parsed document model

- STATUS: CLOSED
- PRIORITY: 125
- TAGS: refactor,v0.1.6

The parsed `Document` stores headings twice (`blocks` and `headings`), recomputes anchors on demand, uses byte offsets as identity and stores images as links.

- Every heading carries its anchor, computed once while parsing. `Document.headings` goes away.
- A block's extent is one `Span` type instead of a `Position` plus a bare `end`.
- Images are kept apart from links, so an image no longer triggers MDS014/MDS015 or counts as a component reference; its target is still checked (MDS012/MDS013).
- No other diagnostic changes.
