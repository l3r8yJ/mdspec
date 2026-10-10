# Add toc_allowed flag to accept a [TOC] marker

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: enhancement
- GITHUB: #15

## Problem

GitLab renders a standalone `[TOC]` paragraph as a table of contents. mdspec 0.1.1 parses it as a shortcut reference link and reports:

```
readme.md:3:1
ERROR MDS011: Undefined reference identifier: TOC
```

A service README with 119 endpoints in one document needs the table of contents, so removing `[TOC]` is not an option, and defining a dummy `[TOC]: #` reference would break GitLab's rendering.

## Proposal

Add a config flag:

```toml
[references]
toc_allowed = true
```

When `true`, a paragraph consisting only of `[TOC]` (also `[[_TOC_]]`, GitLab's other spelling) is treated as a table-of-contents marker, not a reference: MDS011 is not reported for it, and it does not count as a link that requires the references section (MDS015).

Default `false`, so current behavior is unchanged.

## Acceptance

- With `toc_allowed = true`, a document whose only undefined reference is a standalone `[TOC]` passes, including with `--strict`.
- With `toc_allowed = true`, `[TOC]` inside a sentence (`see [TOC] below`) is still a reference and still reports MDS011.
- With `toc_allowed = false` (default), `[TOC]` reports MDS011 as today.
