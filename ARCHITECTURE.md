# Architecture

mdspec reads Markdown files, builds a model of each one, and reports rule violations. It never edits files or touches the network. `src/main.rs` wires the stages together.

## Pipeline

1. `workspace::discover` (`src/workspace.rs`) turns inputs and `--exclude` patterns into a list of files. Bad inputs become MDS901 and MDS902 diagnostics.
2. `parser::parse` (`src/parser.rs`) turns one file into a `Document` (`src/model.rs`): blocks with spans, links, link definitions, and nested headings.
3. `outline::build` (`src/outline.rs`) groups the blocks into an `Outline`: preamble, parts, endpoints with their sections and items, general components, references sections, and stray headings. It takes the configured labels, so it knows which heading is which.
4. `rules::check` (`src/rules/mod.rs`) runs every rule family over all `Source` values (path, document, outline) and collects `Diagnostic` values.
5. `reporter` (`src/reporter.rs`) builds a `Report`, sorts the diagnostics, prints them as text or JSON, and picks the exit code.

`main.rs` also loads `mdspec.toml` (`src/config.rs`) and reports its errors as MDS900. It reports a file that is not UTF-8 as MDS901. `--strict` and `warnings_as_errors` raise all severities to errors before the report is built.

## Rules by file

Each rule ID is emitted by exactly one file in `src/rules/`.

| Rules | File | Subject |
| --- | --- | --- |
| MDS001, MDS005, MDS006 | `structure.rs` | document structure |
| MDS002 | `path.rs` | endpoint path |
| MDS003, MDS004, MDS007 | `sections.rs` | endpoint sections and the general Components section |
| MDS008, MDS009 | `components.rs` | components |
| MDS010 | `mappings.rs` | mapping tables |
| MDS011 to MDS016 | `references.rs` | references and links |
| MDS017 | `examples.rs` | examples |
| MDS018, MDS019 | `component_scope.rs` | component scope |
| MDS900 to MDS902 | `main.rs`, `workspace.rs` | configuration, unreadable input, nothing to check |

`src/links.rs` resolves local links and is shared by `references.rs` and `component_scope.rs`. Rule IDs are defined in `src/rule.rs`.

## Where structure lives

Which part or section a heading belongs to is decided by the outline, not by rules. Rules read a `Source` (path, document, outline) and take structure only from the outline; `references.rs` and `component_scope.rs` also read document links and definitions. Rules report. Heading levels are named constants in `outline::level`. Do not write a bare heading level in a rule.

## Adding a rule

1. Add the variant to `src/rule.rs`.
2. Document it under `## Rules` in the README. The README examples are tested by `tests/readme.rs`.
3. Put the check in the family file that owns its subject. For a new subject, add `src/rules/<family>.rs` with `pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic>` and list it in `CHECKS` in `src/rules/mod.rs`.
4. Test it from the outside in the matching file under `tests/`, with `Workspace::check_rules` from `tests/common/mod.rs`.
5. Run `just full`.

Diagnostics are sorted in `Report::new`, so the order of families does not matter.
