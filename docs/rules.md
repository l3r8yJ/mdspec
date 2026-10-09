# Rule catalog

Labels in examples use an English `[labels]` configuration: `Endpoint:`, `Path:`, `Logic`, `Components`, `Mappings`, `References`, `Description`, `Source`, `Target`. These are configurable vocabulary, not reserved English words. Omitted configuration retains the original Russian defaults. Each fragment assumes an otherwise valid document.

Rules MDS001–MDS015 are errors. MDS016 starts as a warning and is promoted to an error by `--strict`. MDS900–MDS902 are operational errors with exit code 2. Document errors produce exit code 1.

| ID | Trigger / expected diagnostic | Valid example | Invalid example |
|---|---|---|---|
| MDS001 | Missing required H2, invalid prefix, empty name, or multiple H2s | `## Endpoint: List` | `## List` → endpoint heading error |
| MDS002 | Missing, duplicate, malformed, or misplaced path metadata | H2 followed by `Path: GET /items` | `Path: FETCH /items` → path format error |
| MDS003 | A section required by configuration is absent | `### Logic` | No logic section → missing required section |
| MDS004 | Duplicate section, or incorrect order when enforcement is enabled | Logic → Components → Mappings → References | Mappings → Logic → section order error |
| MDS005 | Invalid heading depth/parent or heading inside list/blockquote | H4 component → H5 Description | H6 Description → nesting error |
| MDS006 | Unknown H3 when `allow_unknown_sections = false` | `### Components` | `### Additional` → unknown section |
| MDS007 | Present logic has no content, or components/mappings have no H4 entries | Logic with a paragraph | Logic immediately followed by another H3 → empty section |
| MDS008 | Component description missing, repeated, or empty | H4 Check → H5 Description → paragraph | H4 Check → H4 Fetch → description error |
| MDS009 | Duplicate trimmed component name within a section | H4 Check, H4 Fetch | Two H4 Check headings → duplicate component |
| MDS010 | Table outside a named mapping, no direct table, missing required columns, no data rows, or empty required cells | Source/Target table with `id`/`itemId` row | Empty Target cell → mapping table error |
| MDS011 | Undefined reference identifier | `[text][key]` with `[key]: target.md` | `[text][missing]` → undefined reference |
| MDS012 | Local target missing, inaccessible, not a file, or invalid URL encoding | `[key]: ../shared/target.md` with an accessible file | `[key]: absent.md` → target file error |
| MDS013 | Missing Markdown anchor or invalid fragment encoding | `target.md#check` with a Check heading | `target.md#absent` → anchor not found |
| MDS014 | Inline local link when reference style is required | `[text][key]` | `[text](target.md)` → reference-style required |
| MDS015 | Missing references section; when placement is enforced, definitions outside its terminal block or other content in it | References followed by definitions at document end | Definition before Logic → definition placement error |
| MDS016 | Unused definition detected in strict mode | `[key]` with `[key]: target.md` | Definition alone → unused definition |
| MDS900 | Configuration cannot be read/parsed, or configured labels are invalid/ambiguous | Distinct nonempty section labels | `logic = 'Same'`, `components = 'Same'` → invalid labels |
| MDS901 | Input path, traversal, or UTF-8 reading failure | Accessible UTF-8 `.md` | Missing input file or byte 0xff → input read error |
| MDS902 | No suitable input Markdown files | Directory containing `.md` | Empty directory → no Markdown files found |

The endpoint path is recognized only in the preamble before the first H3. Later component prose may start with the path prefix without becoming metadata. Disabling `require_endpoint` allows H3 sections without H2/Path; an existing H2 remains subject to validation.

An empty document produces at least MDS001, MDS002, and MDS003 under default configuration. Diagnostics accumulate instead of stopping at the first violation. IDs and JSON fields are stable under schema version 1; human-readable messages may change. Configured labels are included in relevant messages.
