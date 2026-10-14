# Build a typed outline of each document

- STATUS: OPEN
- PRIORITY: 120
- TAGS: refactor,v0.2.0

Every rule rebuilds the document structure (endpoint, section, component, description) from heading levels, label comparisons and byte offsets.

- Add an outline built once per document from the parsed model and the labels: title, endpoints with their path and sections, the general components section, the references section, and headings that fit nowhere.
- Heading levels and label matching live only in the outline builder.
- The structural rules that the builder already sees (MDS001, MDS004 placement, MDS005, MDS006) are reported from it.
- No diagnostic changes.
