use crate::config::Config;
use crate::diagnostics::{Diagnostic, Severity};
use crate::links::{self, Destination, percent_decode};
use crate::model::{Block, BlockKind, Document, Link, LinkKind, Position};
use crate::parser;
use crate::rule::Rule;
use crate::rules::Source;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

type TargetCache = HashMap<PathBuf, Result<HashSet<String>, String>>;

struct Validation<'a> {
    source: &'a Source<'a>,
    config: &'a Config,
    cache: TargetCache,
    diagnostics: Vec<Diagnostic>,
}

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    sources
        .iter()
        .flat_map(|source| {
            let mut validation = Validation {
                source,
                config,
                cache: HashMap::new(),
                diagnostics: Vec::new(),
            };
            validation.placement();
            validation.links();
            validation.definitions();
            validation.diagnostics
        })
        .collect()
}

impl Validation<'_> {
    fn error(&mut self, rule: Rule, position: Option<Position>, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(rule, self.source.path, position, message));
    }

    fn links(&mut self) {
        for link in checked_links(self.source.document, self.config) {
            match (&link.key, &link.url) {
                (Some(key), None) => self.error(
                    Rule::Mds011,
                    Some(link.position),
                    format!("Undefined reference identifier: {key}"),
                ),
                (None, Some(url)) => {
                    if link.kind == LinkKind::Link
                        && self.config.references.require_reference_style
                        && links::is_local(url)
                    {
                        self.error(
                            Rule::Mds014,
                            Some(link.position),
                            "Local links must use reference-style syntax",
                        );
                    }
                    self.target(url, link.position);
                }
                _ => {}
            }
        }
    }

    fn definitions(&mut self) {
        let used: HashSet<&str> = self
            .source
            .document
            .links
            .iter()
            .filter_map(|link| link.key.as_deref())
            .collect();
        for definition in &self.source.document.definitions {
            if !used.contains(definition.key.as_str()) {
                let mut diagnostic = Diagnostic::error(
                    Rule::Mds016,
                    self.source.path,
                    Some(definition.position),
                    "Reference definition is unused",
                );
                diagnostic.severity = Severity::Warning;
                self.diagnostics.push(diagnostic);
            }
            self.target(&definition.url, definition.position);
        }
    }

    fn placement(&mut self) {
        let source = self.source;
        let sections = &source.outline.references;
        let first_link =
            checked_links(source.document, self.config).find(|link| link.kind == LinkKind::Link);
        if sections.is_empty() && (first_link.is_some() || !source.document.definitions.is_empty())
        {
            let position = first_link.map(|link| link.position).or_else(|| {
                source
                    .document
                    .definitions
                    .first()
                    .map(|definition| definition.position)
            });
            self.error(
                Rule::Mds015,
                position,
                format!(
                    "Documents with links must contain the {} section",
                    self.config.labels.references
                ),
            );
        }
        if !self.config.references.definitions_at_end {
            return;
        }
        for definition in &source.document.definitions {
            if !sections
                .iter()
                .any(|section| definition_is_terminal(section, definition.position.offset))
            {
                self.error(
                    Rule::Mds015,
                    Some(definition.position),
                    format!(
                        "Reference definitions must appear in the {} section at the end of the document",
                        self.config.labels.references
                    ),
                );
            }
        }
        for block in sections
            .iter()
            .flat_map(|section| section.iter())
            .filter(|block| !matches!(block.kind, BlockKind::Definition))
        {
            self.error(
                Rule::Mds015,
                Some(block.span.start),
                format!(
                    "Only reference definitions are allowed after the {} heading",
                    self.config.labels.references
                ),
            );
        }
    }

    fn target(&mut self, url: &str, position: Position) {
        if !self.config.references.validate_local_paths && !self.config.references.validate_anchors
        {
            return;
        }
        match links::resolve(url, self.source.path) {
            Destination::External => {}
            Destination::InvalidPath => self.error(
                Rule::Mds012,
                Some(position),
                "Invalid percent encoding in local path",
            ),
            Destination::Local {
                file,
                same_file,
                fragment,
            } => {
                if self.config.references.validate_local_paths
                    && !same_file
                    && let Err(error) = check_file(&file)
                {
                    self.error(Rule::Mds012, Some(position), error);
                    return;
                }
                if same_file
                    || file
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
                {
                    self.fragment(&file, fragment, position);
                }
            }
        }
    }

    fn fragment(&mut self, target: &Path, fragment: Option<&str>, position: Position) {
        let Some(fragment) = fragment.filter(|fragment| !fragment.is_empty()) else {
            return;
        };
        if !self.config.references.validate_anchors {
            return;
        }
        let Some(fragment) = percent_decode(fragment) else {
            self.error(
                Rule::Mds013,
                Some(position),
                "Invalid percent encoding in link fragment",
            );
            return;
        };
        let target_anchors = if target == self.source.path {
            Ok(anchors(self.source.document))
        } else {
            target_anchors(target, &mut self.cache)
        };
        match target_anchors {
            Ok(anchors) if !anchors.contains(&fragment) => self.error(
                Rule::Mds013,
                Some(position),
                format!("Anchor #{fragment} not found in {}", target.display()),
            ),
            Err(error) => self.error(Rule::Mds012, Some(position), error),
            _ => {}
        }
    }
}

fn checked_links<'a>(document: &'a Document, config: &'a Config) -> impl Iterator<Item = &'a Link> {
    document
        .links
        .iter()
        .filter(|link| !(link.table_of_contents && config.references.toc_allowed))
}

fn definition_is_terminal(section: &[&Block], offset: usize) -> bool {
    section
        .iter()
        .all(|block| matches!(block.kind, BlockKind::Definition))
        && section
            .iter()
            .any(|block| block.span.start.offset == offset)
}

fn check_file(target: &Path) -> Result<(), String> {
    let metadata = fs::metadata(target)
        .map_err(|error| format!("Target file not found {}: {error}", target.display()))?;
    if !metadata.is_file() {
        return Err(format!("Link target is not a file: {}", target.display()));
    }
    fs::File::open(target)
        .map_err(|error| format!("Unable to open {}: {error}", target.display()))?;
    Ok(())
}

fn anchors(document: &Document) -> HashSet<String> {
    document.anchors().map(str::to_owned).collect()
}

fn target_anchors(target: &Path, cache: &mut TargetCache) -> Result<HashSet<String>, String> {
    cache
        .entry(target.to_path_buf())
        .or_insert_with(|| {
            let source = fs::read_to_string(target)
                .map_err(|error| format!("Unable to read {}: {error}", target.display()))?;
            Ok(anchors(&parser::parse(&source)))
        })
        .clone()
}
