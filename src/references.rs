use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::diagnostics::{Diagnostic, Severity};
use crate::model::{BlockKind, Document, Position};
use crate::parser;

pub fn validate(
    document: &Document,
    path: &Path,
    config: &Config,
    strict: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut cache = HashMap::new();
    let used: HashSet<&str> = document
        .links
        .iter()
        .filter_map(|link| link.key.as_deref())
        .collect();
    validate_placement(document, path, config, &mut diagnostics);
    for link in &document.links {
        match (&link.key, &link.url) {
            (Some(key), None) => diagnostics.push(Diagnostic::error(
                "MDS011",
                path,
                Some(link.position),
                format!("Undefined reference identifier: {key}"),
            )),
            (None, Some(url)) => {
                if config.references.require_reference_style && is_local(url) {
                    diagnostics.push(Diagnostic::error(
                        "MDS014",
                        path,
                        Some(link.position),
                        "Local links must use reference-style syntax",
                    ));
                }
                validate_target(
                    url,
                    link.position,
                    document,
                    path,
                    config,
                    &mut cache,
                    &mut diagnostics,
                );
            }
            _ => {}
        }
    }
    for definition in &document.definitions {
        if strict && !used.contains(definition.key.as_str()) {
            let mut diagnostic = Diagnostic::error(
                "MDS016",
                path,
                Some(definition.position),
                "Reference definition is unused",
            );
            diagnostic.severity = Severity::Warning;
            diagnostics.push(diagnostic);
        }
        validate_target(
            &definition.url,
            definition.position,
            document,
            path,
            config,
            &mut cache,
            &mut diagnostics,
        );
    }
    diagnostics
}

fn validate_placement(
    document: &Document,
    path: &Path,
    config: &Config,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let references = document.blocks.iter().position(|block| {
        matches!(block.kind, BlockKind::Heading(3)) && block.text.trim() == config.labels.references
    });
    if (!document.links.is_empty() || !document.definitions.is_empty()) && references.is_none() {
        diagnostics.push(Diagnostic::error(
            "MDS015",
            path,
            document
                .links
                .first()
                .map(|link| link.position)
                .or_else(|| {
                    document
                        .definitions
                        .first()
                        .map(|definition| definition.position)
                }),
            format!(
                "Documents with links must contain the {} section",
                config.labels.references
            ),
        ));
    }
    if !config.references.definitions_at_end {
        return;
    }
    for definition in &document.definitions {
        let in_section = references.is_some_and(|index| {
            document.blocks[index].position.offset < definition.position.offset
                && !document.blocks[index + 1..].iter().any(|block| {
                    matches!(block.kind, BlockKind::Heading(1..=3))
                        && block.position.offset < definition.position.offset
                })
        });
        let is_root_definition = document.blocks.iter().any(|block| {
            matches!(block.kind, BlockKind::Definition)
                && block.position.offset == definition.position.offset
        });
        let followed_by_content = document.blocks.iter().any(|block| {
            block.position.offset > definition.position.offset
                && !matches!(block.kind, BlockKind::Definition)
        });
        if !in_section || !is_root_definition || followed_by_content {
            diagnostics.push(Diagnostic::error(
                "MDS015",
                path,
                Some(definition.position),
                format!("Reference definitions must appear in the {} section at the end of the document", config.labels.references),
            ));
        }
    }
    if let Some(index) = references {
        for block in &document.blocks[index + 1..] {
            if !matches!(block.kind, BlockKind::Definition) {
                diagnostics.push(Diagnostic::error(
                    "MDS015",
                    path,
                    Some(block.position),
                    format!(
                        "Only reference definitions are allowed after the {} heading",
                        config.labels.references
                    ),
                ));
            }
        }
    }
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
            let high = char::from(bytes.next()?).to_digit(16)?;
            let low = char::from(bytes.next()?).to_digit(16)?;
            decoded.push((high * 16 + low) as u8);
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

type TargetCache = HashMap<PathBuf, Result<HashSet<String>, String>>;

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

fn validate_target(
    url: &str,
    position: Position,
    document: &Document,
    path: &Path,
    config: &Config,
    cache: &mut TargetCache,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !is_local(url)
        || (!config.references.validate_local_paths && !config.references.validate_anchors)
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
        diagnostics.push(Diagnostic::error(
            "MDS012",
            path,
            Some(position),
            "Invalid percent encoding in local path",
        ));
        return;
    };
    let target = if decoded_path.is_empty() {
        path.to_path_buf()
    } else {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&decoded_path)
    };
    if config.references.validate_local_paths && !decoded_path.is_empty() {
        match fs::metadata(&target) {
            Ok(metadata) if metadata.is_file() => {
                if let Err(error) = fs::File::open(&target) {
                    diagnostics.push(Diagnostic::error(
                        "MDS012",
                        path,
                        Some(position),
                        format!("Unable to open {}: {error}", target.display()),
                    ));
                    return;
                }
            }
            Ok(_) => {
                diagnostics.push(Diagnostic::error(
                    "MDS012",
                    path,
                    Some(position),
                    format!("Link target is not a file: {}", target.display()),
                ));
                return;
            }
            Err(error) => {
                diagnostics.push(Diagnostic::error(
                    "MDS012",
                    path,
                    Some(position),
                    format!("Target file not found {}: {error}", target.display()),
                ));
                return;
            }
        }
    }
    let Some(fragment) = fragment.filter(|fragment| !fragment.is_empty()) else {
        return;
    };
    if !config.references.validate_anchors
        || (!decoded_path.is_empty()
            && !target
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md")))
    {
        return;
    }
    let Some(fragment) = percent_decode(fragment) else {
        diagnostics.push(Diagnostic::error(
            "MDS013",
            path,
            Some(position),
            "Invalid percent encoding in link fragment",
        ));
        return;
    };
    let target_anchors = if decoded_path.is_empty() {
        Ok(anchors(document))
    } else {
        target_anchors(&target, cache)
    };
    match target_anchors {
        Ok(anchors) if !anchors.contains(&fragment) => diagnostics.push(Diagnostic::error(
            "MDS013",
            path,
            Some(position),
            format!("Anchor #{fragment} not found in {}", target.display()),
        )),
        Err(error) => diagnostics.push(Diagnostic::error("MDS012", path, Some(position), error)),
        _ => {}
    }
}
