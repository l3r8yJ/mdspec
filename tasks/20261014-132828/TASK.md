# Move each rule family onto the outline

- STATUS: OPEN
- PRIORITY: 115
- TAGS: refactor,v0.2.0

With the outline in place, rule checks still live in one 600-line `rules.rs` plus `references.rs` and `components.rs`, and several rule IDs are emitted from more than one place.

- One file per rule family under `src/rules/`, each reading the outline and returning its diagnostics through one shared function shape.
- Each rule ID is emitted from exactly one module.
- One link resolver shared by MDS012/MDS013 and MDS018/MDS019.
- An `ARCHITECTURE.md` explains the pipeline (parse, outline, rules, report) and where a new rule goes.
- No diagnostic changes.
