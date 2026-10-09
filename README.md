# mdspec

A CLI validator for structured Markdown endpoint documentation. Checks document structure, reusable components, mapping tables, and local references. It does not evaluate business logic, modify documents, or access external URLs.

## Build and run

Requires stable Rust 1.96.0, pinned in `rust-toolchain.toml`.

Published versions can be installed from crates.io:

```sh
cargo install mdspec --locked
mdspec --version
```

Cargo downloads the source package and compiles the executable for the user's system. GitHub Releases separately provides prebuilt Linux binaries. Before the first crates.io publication, or to build from a checkout:

```sh
cargo build --release
cargo install --path . --locked
mdspec check docs/
mdspec check docs/example.md --strict --format json
mdspec check docs/ --config mdspec.toml
mdspec --help
mdspec --version
```

Accepts one file or directory. Recursively discovers `.md` files, case-insensitively; respects hidden files, `.ignore`, `.gitignore`, and global Git ignores through `ignore`, including outside Git repositories. Directory traversal does not follow symlinks. Every discovered Markdown file is validated as an endpoint document. Referenced files outside the input are read for anchors, without validating their endpoint structure.

## Documents in any language

The schema has fixed structural roles, but **every required label is configurable**. No language detection or built-in translation pack is required. Use `[labels]` in your TOML configuration:

```toml
[labels]
endpoint_prefix = "Endpoint:"
path_prefix = "Path:"
logic = "Logic"
components = "Components"
mappings = "Mappings"
references = "References"
description = "Description"
source = "Source"
target = "Target"
```

With that configuration, this is a minimal valid document:

```md
## Endpoint: List backups

Path: `GET /v1/backups:all`

### Logic

1. Return the backups.
```

Labels can use any Unicode script. Matching is case-sensitive against parsed Markdown text. Prefixes include their punctuation; `Endpoint:` and `Endpoint：` are distinct. Labels must be nonempty, single-line strings without surrounding whitespace. The four section names must be distinct, as must the source and target column labels. Invalid labels produce MDS900 before document validation.

For backward compatibility, omitted labels retain the original defaults: `Эндпоинт:`, `Path:`, `Логика работы`, `Компоненты`, `Маппинги`, `Ссылки`, `Описание`, `Source`, `Target`. A partial `[labels]` table overrides only its supplied fields. Configuring labels changes the required vocabulary; it does not silently accept alternate translations. User-authored prose, component names, and mapping names may mix languages freely.

Complete executable examples:

```sh
mdspec check tests/fixtures/languages/english.md --config tests/fixtures/languages/english.toml --strict
mdspec check tests/fixtures/languages/japanese.md --config tests/fixtures/languages/japanese.toml --strict
```

CLI messages are in English; diagnostics quote configured labels when relevant. Rule IDs and JSON fields are language-independent. Custom labels do not alter reference identifier normalization or anchor generation.

## Structural contract

- Exactly one H2 endpoint heading with the configured prefix and a nonempty name.
- Its next block is a separate path paragraph: configured prefix, HTTP method, and path beginning with `/` without whitespace. Inline code is optional. Supported methods: GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS, CONNECT, TRACE. Path-like prose inside later sections is not endpoint metadata.
- H3 sections follow logic → components → mappings → references. Logic is required and nonempty; lists are optional. Components and mappings may be omitted unless configured as required.
- Each H4 component has a unique trimmed name and exactly one nonempty H5 description with the configured label.
- Each H4 mapping directly contains a table with the configured source and target columns, at least one data row, and nonempty required cells. Extra columns are allowed. For repeated column names, the first occurrence is checked.
- Structural headings inside lists or blockquotes are invalid. Code contents do not create headings or links. HTML comments do not count as content.
- Any links or definitions require the references section. By default, it contains only definitions, at the end of the document. External inline links are allowed; local links and images must use reference style.

See the full default configuration in [mdspec.toml](mdspec.toml).

## Rules

Examples use the English labels above and show fragments of an otherwise valid document. `→` means “followed by”.

| Rule | Correct example | Incorrect example |
|---|---|---|
| MDS001 — Endpoint heading | `## Endpoint: List items` | `## List items` |
| MDS002 — HTTP path | `Path: GET /items` immediately after H2 | `Path: FETCH /items` |
| MDS003 — Required sections | `### Logic` present | No logic section |
| MDS004 — Section order and uniqueness | Logic → Components → Mappings → References | Mappings → Logic, or two Logic sections |
| MDS005 — Heading levels | `#### Check` → `##### Description` | `#### Check` → `###### Description` |
| MDS006 — Known sections (when enforced) | `### Components` | `### Extra` |
| MDS007 — Nonempty sections | `### Logic` → `1. Return items.` | `### Logic` → next H3 |
| MDS008 — Component description | `#### Check` → `##### Description` → `Check access.` | Description missing or empty |
| MDS009 — Unique component names | `#### Check` → `#### Fetch` | Two `#### Check` headings |
| MDS010 — Mapping table under H4 | `Source \| Target`<br>`--- \| ---`<br>`id \| itemId` | No table, missing column, or empty required cell |
| MDS011 — Defined reference keys | `[items][key]` with `[key]: items.md` | `[items][missing]` without a definition |
| MDS012 — Existing local file | `[key]: items.md` where the file exists | `[key]: missing.md` |
| MDS013 — Existing anchor | `[key]: items.md#check` with `## Check` in the target | `[key]: items.md#missing` |
| MDS014 — Reference-style local links | `[items][key]` with a definition | `[items](items.md)` |
| MDS015 — Definitions at the end | Final `### References` → definitions only | Definition before Logic |
| MDS016 — Used definitions (strict mode) | `[key]` plus its definition | Definition with no usage |

MDS001–MDS015 are errors. MDS016 is enabled by `--strict` and is promoted to an error. Configuration can relax selected rules; see below.

## References and anchors

Full, collapsed, and shortcut reference links are supported. CommonMark handles Unicode case folding and whitespace normalization of reference identifiers: for example, `Straße` and `STRASSE` match. Undefined `[word]` is reported as an unresolved reference; escape literal brackets or put them in code.

Heading anchors use a separate fixed MVP contract: lowercase heading text, replace each whitespace character with `-`, retain Unicode letters/numbers and `_`/`-`, and remove other characters. HTML tags are excluded. Collisions receive `-1`, `-2`, accounting for previously occupied names. Fragments are case-sensitive. There is no NFC normalization; combining marks and emoji are removed. This does not promise exact compatibility with every GitHub/GitLab renderer.

Relative paths resolve from the source document's directory; `..` is allowed. Absolute paths are filesystem-absolute. Path and fragment are percent-decoded separately; `+` remains literal. Query strings do not affect file lookup. `#anchor` targets the current document; an empty fragment means the start of a file.

All definition targets are checked, including unused definitions. URI schemes and protocol-relative URLs are skipped without network access. Non-Markdown files are checked for existence and accessibility; their fragments are not validated.

## Configuration

Only an explicit `--config` file is read; otherwise built-in defaults apply. Missing fields use defaults; unknown fields and invalid types are configuration errors.

`document.require_endpoint = false` permits H3 sections without H2/Path; an existing H2 is still validated. Component-only documentation also needs `require_logic = false`. `references.require_reference_style = false` allows inline links while retaining target checks. `references.definitions_at_end = false` relaxes definition placement and reference-section contents; the section itself remains required when links exist.

`--strict` enables MDS016 for unused definitions and promotes warnings to errors. `diagnostics.warnings_as_errors` promotes emitted warnings without enabling strict rules. MDS016 is currently the only rule with an underlying warning severity.

## Diagnostics and CI

Text reports go to stderr. JSON goes exclusively to stdout, sorted by file, line, column, rule ID, and message. Use IDs for automation; message text can change.

Exit codes:

- `0`: no errors.
- `1`: document violations, including broken references.
- `2`: argument, configuration, input traversal/read errors, or no Markdown files. Takes priority over `1` when failures are mixed.

Processing continues after individual file failures. A closed output pipe is handled without a panic.

JSON schema version 1 has these stable fields:

```json
{
  "schema_version": 1,
  "files_checked": 1,
  "error_count": 1,
  "warning_count": 0,
  "diagnostics": [
    {
      "rule": "MDS003",
      "severity": "error",
      "file": "docs/example.md",
      "line": null,
      "column": null,
      "message": "Missing required section: Logic",
      "help": null
    }
  ]
}
```

All fields are present. Severity is `error` or `warning`. Positions are one-based Unicode scalar columns, not byte offsets or display widths; unavailable positions are `null`. Help is a string or `null`. `files_checked` counts successfully read UTF-8 documents regardless of violations. Paths need not be canonical. Counts reflect severity after warning promotion.

Configuration and filesystem failures also produce JSON. Clap argument errors use text on stderr and exit `2`; stdout is empty. Help and version output are always text.

## Contributing

Install the development tools once:

```sh
cargo install just --version 1.58.0 --locked
cargo install cargo-llvm-cov --version 0.9.1 --locked
rustup component add llvm-tools-preview
```

The `justfile` is shared by local development and CI:

```sh
just fmt
just fmt-check
just clippy
just test
just coverage
just build
just package
just publish-check
just ci
```

Tests drive real CLI processes and fixtures, covering valid and invalid documents, multiple simultaneous violations, Unicode, code blocks, cross-file references, custom English/Japanese labels, strict/config behavior, invalid UTF-8, output formats, exit codes, and directory traversal.

`just coverage` runs the tests once with LLVM instrumentation, then fails if total line coverage is below **70%**. Integration-test sources and dependencies are excluded by cargo-llvm-cov's default filters. CLI subprocesses contribute coverage. `target/coverage/lcov.info` contains the report. `just ci` runs formatting, strict linting, tests with coverage, binary packaging, and a crates.io publication dry run; it does not also run the standalone `just test` recipe. `just publish-check` builds the packaged source to verify that it is self-contained, without uploading it.

Clippy enables `all`, `pedantic`, and `nursery`, with all warnings treated as errors. `unwrap_used`, `expect_used`, `indexing_slicing`, and `panic` are denied. The exact settings in `clippy.toml` allow `expect` and indexing in tests but do not allow explicit panics there; production code must handle those operations safely. Cognitive complexity is limited to 10, arguments to 4, and function length to 80 lines. The sole group exception is `struct_excessive_bools`: independent TOML configuration switches intentionally remain booleans.

## Limits

CommonMark accepts almost any text; an unclosed code fence is not itself a syntax error. Schema violations are still checked. Repeated reference definitions use CommonMark first-wins without a separate duplicate diagnostic. YAML front matter, wiki links, footnotes, explicit HTML anchors, autofixes, plugins, and watch mode are outside the MVP. A single run uses one label configuration for all input documents; mixed structural vocabularies require separate runs.
