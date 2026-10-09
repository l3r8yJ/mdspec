# MVP architecture

One synchronous binary crate separates parsing, validation, reference resolution, and reporting. There is no plugin framework, async runtime, or duplicated domain hierarchy.

## Parser choice

APIs were checked against primary documentation and downloaded crate sources; compatibility was verified by building.

| Parser | Structure, tables, positions | Reference definitions | Undefined references |
|---|---|---|---|
| markdown-rs (`markdown 1.0.0`) | mdast via `to_mdast` and `ParseOptions::gfm`, byte offsets | Definition/LinkReference nodes | Become Text; no callback |
| comrak 0.56.0 | Arena AST, sourcepos, GFM tables | Private refmap, absent from AST | Broken-link callback |
| **pulldown-cmark 0.13.4** | Nested Start/End events, tables, byte ranges | `reference_definitions()` with destinations and spans | Callback with Unknown reference variants |

**pulldown-cmark** supplies all required information in one parser. Combining markdown-rs with comrak would require two different parsing passes; using comrak alone would require parsing definitions ourselves. The preference for markdown-rs was overridden because undefined-reference diagnostics are mandatory.

Sources: [markdown-rs](https://github.com/wooorm/markdown-rs), [mdast](https://docs.rs/markdown/1.0.0/markdown/mdast/index.html), [comrak options](https://docs.rs/comrak/0.56.0/comrak/options/struct.Parse.html), [comrak NodeValue](https://docs.rs/comrak/0.56.0/comrak/nodes/enum.NodeValue.html), [pulldown Parser](https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/struct.Parser.html).

## Data flow

`cli → config + workspace → parser → rules + references → reporter`

- `config.rs` defines validation switches and configurable schema labels. Defaults preserve existing documents. Labels are checked at the configuration boundary: nonempty, single-line, trimmed, with unique section names and different source/target columns. Validation is language-independent; English diagnostic templates can include user-configured vocabulary.
- `parser.rs` adapts the parser's event tree into root blocks, tables, headings, definitions, and link uses. It does not implement Markdown syntax. A line-start index converts byte offsets to Unicode positions. Nested headings are tracked separately; code text does not create headings or links.
- `model.rs` contains CLI-independent data. Sections end at the next root heading of equal or shallower depth. No duplicate full AST is retained.
- `rules.rs` accumulates MDS001–MDS010 using the shared block index and configured labels.
- `references.rs` handles MDS011–MDS016. Reference identity comes from CommonMark normalization, independently of heading-anchor generation. Target files are parsed only for headings; anchor sets are cached for each source document's validation. Cycles do not trigger recursive validation.
- `workspace.rs` uses `ignore`, sorts inputs, and preserves traversal failures. It does not follow directory symlinks.
- `diagnostics.rs` provides serializable diagnostics and `miette::Diagnostic`; `reporter.rs` emits text or versioned JSON.
- `main.rs` integrates results: operational errors use exit 2; document violations use exit 1. Input documents are never modified and validation makes no network requests.

One invocation uses one vocabulary configuration. Supporting mixed schema vocabularies within a directory would require an explicit selection mechanism; automatic language detection is deliberately absent.

## Dependencies

Cargo.lock pins clap 4.6.7, pulldown-cmark 0.13.4, miette 7.6.0, serde 1.0.229, serde_json 1.0.151, toml 0.9.12, and ignore 0.4.33. Tests use asserting and tempfile. The lockfile is part of the CLI distribution.
