# mdspec

Validates Markdown endpoint documentation: structure, components, mapping tables, and local links. Never edits files or touches the network.

## Install

```sh
cargo install mdspec --locked
```

Prebuilt x86_64 Linux binaries are on [GitHub Releases](https://github.com/l3r8yJ/mdspec/releases). The `x86_64-unknown-linux-gnu` archive needs glibc; the statically linked `x86_64-unknown-linux-musl` archive also runs on Alpine and other images without glibc.

## Quick start

The default headings are Russian (see [mdspec.toml](mdspec.toml)). For English docs, save this as `mdspec.toml`:

```toml
[labels]
endpoint_prefix = "Endpoint:"
path_prefix = "Path:"
logic = "Logic"
components = "Components"
mappings = "Mappings"
examples = "Examples"
references = "References"
description = "Description"
source = "Source"
target = "Target"
request = "Request"
response = "Response"
```

Write a document:

```md
## Endpoint: List items

Path: `GET /v1/items`

### Logic

1. Return the items.
```

Check it:

```sh
mdspec check docs/ --config mdspec.toml
```

Labels can be in any language. Unset labels keep their defaults.

## Usage

```sh
mdspec check docs/
mdspec check docs/a.md docs/b.md
mdspec check 'docs/*/service/*.md'
mdspec check 'docs/**/*.md' --exclude 'docs/**/generated/**'
mdspec check docs/ --strict --format json
```

- Quote globs. `*` stays within one folder, `**` crosses folders.
- `--exclude` is repeatable and always wins.
- Folders are scanned for `.md` files, respecting `.gitignore` and `.ignore`.
- `--strict` reports unused link definitions and turns warnings into errors.
- `--format json` prints a report to stdout. Match on `rule` IDs, not messages.

Exit codes:

- `0` clean
- `1` rule violations
- `2` bad arguments, config, or input (wins over `1`)

## Configuration

Only a file passed with `--config` is read. Unknown keys are errors. All keys and defaults: [mdspec.toml](mdspec.toml).

- `document.require_endpoint = false` allows documents without the endpoint heading and path.
- `document.allow_unknown_sections = false` rejects sections not listed in `[labels]`.
- `document.require_examples = true` requires the examples section.
- `document.multiple_endpoints = false` allows exactly one endpoint per document. By default every `## <endpoint_prefix> name` heading starts an endpoint that runs to the next H2. Section and component rules apply per endpoint. An optional H1 title and other H2 sections such as general rules are not checked. Link definitions go in each endpoint's references section or in one H2 references section at the end of the document.
- `references.require_reference_style = false` allows inline local links.
- `references.definitions_at_end = false` allows definitions anywhere.
- `references.toc_allowed = true` accepts a paragraph that is only `[TOC]` or `[[_TOC_]]` as a table of contents marker instead of an undefined reference.

## Rules

Examples use the English labels from Quick start.

### MDS001 Endpoint heading

Correct:

```md
## Endpoint: List items
```

Incorrect:

```md
## List items
```

### MDS002 HTTP path

Correct:

```md
## Endpoint: List items

Path: `GET /v1/items`
```

Incorrect:

```md
## Endpoint: List items

Path: `FETCH /v1/items`
```

### MDS003 Required sections

Correct:

```md
### Logic

1. Return the items.
```

Incorrect:

```md
### Components
```

### MDS004 Section order

Correct:

```md
### Logic
### Components
### Mappings
### Examples
### References
```

Incorrect:

```md
### Mappings
### Logic
```

### MDS005 Heading levels

Correct:

```md
#### Fetch data

##### Description
```

Incorrect:

```md
#### Fetch data

###### Description
```

### MDS006 Known sections

Only with `allow_unknown_sections = false`.

Correct:

```md
### Components
```

Incorrect:

```md
### Extras
```

### MDS007 Nonempty sections

Correct:

```md
### Logic

1. Return the items.
```

Incorrect:

```md
### Logic

### Components
```

### MDS008 Component description

Correct:

```md
#### Fetch data

##### Description

Read the items.
```

Incorrect:

```md
#### Fetch data

Read the items.
```

### MDS009 Unique component names

Correct:

```md
#### Fetch data
#### Save data
```

Incorrect:

```md
#### Fetch data
#### Fetch data
```

### MDS010 Mapping table

Correct:

```md
#### Item mapping

| Source | Target |
| --- | --- |
| id | itemId |
```

Incorrect:

```md
#### Item mapping

| Source | Name |
| --- | --- |
| id | |
```

### MDS011 Defined references

Correct:

```md
See [items][list].

[list]: items.md
```

Incorrect:

```md
See [items][missing].
```

### MDS012 Existing files

Correct:

```md
[list]: items.md
```

Incorrect:

```md
[list]: no-such-file.md
```

### MDS013 Existing anchors

Correct:

```md
[fetch]: items.md#fetch-data
```

Incorrect:

```md
[fetch]: items.md#no-such-heading
```

### MDS014 Reference-style local links

Correct:

```md
See [items][list].

[list]: items.md
```

Incorrect:

```md
See [items](items.md).
```

### MDS015 Definitions at the end

Correct:

```md
### References

[list]: items.md
```

Incorrect:

```md
[list]: items.md

### Logic
```

### MDS016 Unused definitions

Only with `--strict`.

Correct:

```md
See [items][list].

[list]: items.md
```

Incorrect:

```md
[list]: items.md
```

### MDS017 Examples

Examples are optional. Each one is a request or response H4 with a fenced code block, at most once per endpoint.

Correct:

````md
### Examples

#### Request

```json
{"name": "backup"}
```

#### Response

```json
{"id": 1}
```
````

Incorrect:

````md
### Examples

#### Headers

Accept: application/json
````

### MDS900 Configuration

Correct:

```toml
[labels]
logic = "Logic"
```

Incorrect:

```toml
[labels]
logic = " Logic "
```

### MDS901 Unreadable input

Correct:

```sh
mdspec check docs/
```

Incorrect:

```sh
mdspec check no-such-dir/
```

### MDS902 Nothing to check

Correct:

```sh
mdspec check docs/api.md
```

Incorrect:

```sh
mdspec check docs/notes.txt
```

## Contributing

Tasks live in [tasks/](tasks) in the [tatr](https://github.com/tsoding/tatr) layout: one `tasks/<YYYYMMDD-HHMMSS>/TASK.md` per task, with the time in UTC. Tags are described in [tasks/tags](tasks/tags). Reference the task ID in commits and PRs.

Every task is also tagged with the release that ships it, e.g. `v0.1.3`, and that tag is added to `tasks/tags`. `tatr ls -c :v0.1.3` lists what a release contains. Open tasks get the next version tag when they are planned for it.

Install tools once:

```sh
cargo install just --version 1.58.0 --locked
cargo install cargo-llvm-cov --version 0.9.1 --locked
cargo install cargo-mutants --version 27.1.0 --locked
rustup component add llvm-tools-preview
```

Before opening a PR:

```sh
just fmt
git commit
just full
```

`just full` runs everything CI runs, plus mutation testing. It needs committed files and publishes nothing.
