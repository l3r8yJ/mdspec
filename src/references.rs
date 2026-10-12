use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::diagnostics::{Diagnostic, Severity};
use crate::model::{Block, BlockKind, Document, Link, Position};
use crate::{parser, rules};

type TargetCache = HashMap<PathBuf, Result<HashSet<String>, String>>;

struct Validation<'a> {
    document: &'a Document,
    path: &'a Path,
    config: &'a Config,
    cache: TargetCache,
    diagnostics: Vec<Diagnostic>,
}

pub fn validate(
    document: &Document,
    path: &Path,
    config: &Config,
    strict: bool,
) -> Vec<Diagnostic> {
    let mut validation = Validation {
        document,
        path,
        config,
        cache: HashMap::new(),
        diagnostics: Vec::new(),
    };
    validation.placement();
    validation.links();
    validation.definitions(strict);
    validation.diagnostics
}

impl Validation<'_> {
    fn error(
        &mut self,
        rule: &'static str,
        position: Option<Position>,
        message: impl Into<String>,
    ) {
        self.diagnostics
            .push(Diagnostic::error(rule, self.path, position, message));
    }

    fn links(&mut self) {
        for link in checked_links(self.document, self.config) {
            match (&link.key, &link.url) {
                (Some(key), None) => self.error(
                    "MDS011",
                    Some(link.position),
                    format!("Undefined reference identifier: {key}"),
                ),
                (None, Some(url)) => {
                    if self.config.references.require_reference_style && is_local(url) {
                        self.error(
                            "MDS014",
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

    fn definitions(&mut self, strict: bool) {
        let used: HashSet<&str> = self
            .document
            .links
            .iter()
            .filter_map(|link| link.key.as_deref())
            .collect();
        for definition in &self.document.definitions {
            if strict && !used.contains(definition.key.as_str()) {
                let mut diagnostic = Diagnostic::error(
                    "MDS016",
                    self.path,
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
        let sections = reference_sections(self.document, self.config);
        let first_link = checked_links(self.document, self.config).next();
        if sections.is_empty() && (first_link.is_some() || !self.document.definitions.is_empty()) {
            let position = first_link.map(|link| link.position).or_else(|| {
                self.document
                    .definitions
                    .first()
                    .map(|definition| definition.position)
            });
            self.error(
                "MDS015",
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
        for definition in &self.document.definitions {
            if !sections
                .iter()
                .any(|section| definition_is_terminal(section, definition.position.offset))
            {
                self.error("MDS015", Some(definition.position), format!("Reference definitions must appear in the {} section at the end of the document", self.config.labels.references));
            }
        }
        for block in sections
            .iter()
            .flat_map(|section| section.iter())
            .filter(|block| !matches!(block.kind, BlockKind::Definition))
        {
            self.error(
                "MDS015",
                Some(block.position),
                format!(
                    "Only reference definitions are allowed after the {} heading",
                    self.config.labels.references
                ),
            );
        }
    }

    fn target(&mut self, url: &str, position: Position) {
        if !is_local(url)
            || (!self.config.references.validate_local_paths
                && !self.config.references.validate_anchors)
        {
            return;
        }
        let (resource, fragment) = url
            .split_once('#')
            .map_or((url, None), |(resource, fragment)| {
                (resource, Some(fragment))
            });
        let raw_path = resource
            .split_once('?')
            .map_or(resource, |(resource, _)| resource);
        let Some(decoded_path) = percent_decode(raw_path) else {
            self.error(
                "MDS012",
                Some(position),
                "Invalid percent encoding in local path",
            );
            return;
        };
        let same_file = decoded_path.is_empty();
        let target = if same_file {
            self.path.to_path_buf()
        } else {
            self.path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&decoded_path)
        };
        if self.config.references.validate_local_paths
            && !same_file
            && let Err(error) = check_file(&target)
        {
            self.error("MDS012", Some(position), error);
            return;
        }
        if same_file
            || target
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        {
            self.fragment(&target, fragment, position);
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
                "MDS013",
                Some(position),
                "Invalid percent encoding in link fragment",
            );
            return;
        };
        let target_anchors = if target == self.path {
            Ok(anchors(self.document))
        } else {
            target_anchors(target, &mut self.cache)
        };
        match target_anchors {
            Ok(anchors) if !anchors.contains(&fragment) => self.error(
                "MDS013",
                Some(position),
                format!("Anchor #{fragment} not found in {}", target.display()),
            ),
            Err(error) => self.error("MDS012", Some(position), error),
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

fn reference_sections<'a>(document: &'a Document, config: &Config) -> Vec<&'a [Block]> {
    let multiple = config.document.multiple_endpoints;
    let sections = document
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.text.trim() == config.labels.references)
        .filter_map(|(index, block)| {
            let extent = match block.kind {
                BlockKind::Heading(3) if multiple => 2,
                BlockKind::Heading(3) => 0,
                BlockKind::Heading(2) if multiple => 0,
                _ => return None,
            };
            Some(rules::children(&document.blocks, index, extent))
        });
    if multiple {
        sections.collect()
    } else {
        sections.take(1).collect()
    }
}

fn definition_is_terminal(section: &[Block], offset: usize) -> bool {
    section
        .iter()
        .all(|block| matches!(block.kind, BlockKind::Definition))
        && section.iter().any(|block| block.position.offset == offset)
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

fn is_local(url: &str) -> bool {
    if url.starts_with("//") {
        return false;
    }
    let Some((scheme, _)) = url.split_once(':') else {
        return true;
    };
    !(scheme.starts_with(|character: char| character.is_ascii_alphabetic())
        && scheme.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        }))
}

fn percent_decode(value: &str) -> Option<String> {
    let mut decoded = Vec::with_capacity(value.len());
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = u8::try_from(char::from(bytes.next()?).to_digit(16)?).ok()?;
            let low = u8::try_from(char::from(bytes.next()?).to_digit(16)?).ok()?;
            decoded.push(high * 16 + low);
        } else {
            decoded.push(byte);
        }
    }
    String::from_utf8(decoded)
        .ok()
        .filter(|value| !value.contains('\0'))
}

fn anchors(document: &Document) -> HashSet<String> {
    let mut anchors = HashSet::new();
    for (heading, _) in &document.headings {
        let base: String = heading
            .to_lowercase()
            .chars()
            .filter_map(|character| {
                if character.is_whitespace() {
                    Some('-')
                } else if character.is_alphanumeric() || matches!(character, '_' | '-') {
                    Some(character)
                } else {
                    None
                }
            })
            .collect();
        let mut anchor = base.clone();
        let mut suffix = 0;
        while anchors.contains(&anchor) {
            suffix += 1;
            anchor = format!("{base}-{suffix}");
        }
        anchors.insert(anchor);
    }
    anchors
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
