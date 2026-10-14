use crate::config::{Config, Labels};
use crate::diagnostics::Diagnostic;
use crate::model::{Block, BlockKind, Document, Position};
use crate::rule::Rule;
use std::collections::HashSet;
use std::path::Path;

pub fn validate(document: &Document, path: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    validate_endpoints(&document.blocks, path, config, &mut diagnostics);
    validate_general_components(&document.blocks, path, config, &mut diagnostics);
    diagnostics
}

fn validate_details(
    blocks: &[Block],
    path: &Path,
    labels: &Labels,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (index, block) in blocks.iter().enumerate() {
        if !matches!(block.kind, BlockKind::Heading { level: 3, .. }) {
            continue;
        }
        let children = children(blocks, index, 3);
        if block.text.trim() == labels.components {
            diagnostics.extend(validate_components(children, 4, path, labels));
        } else if block.text.trim() == labels.mappings {
            validate_mappings(children, path, labels, diagnostics);
        } else if block.text.trim() == labels.examples {
            validate_examples(children, path, labels, diagnostics);
        }
    }
}

pub fn children(blocks: &[Block], index: usize, depth: u8) -> &[Block] {
    let rest = blocks.get(index + 1..).unwrap_or_default();
    let length = rest
        .iter()
        .position(|block| matches!(block.kind, BlockKind::Heading { level, .. } if level <= depth))
        .unwrap_or(rest.len());
    rest.get(..length).unwrap_or_default()
}

fn issue(rule: Rule, path: &Path, block: &Block, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(rule, path, Some(block.span.start), message)
}

fn validate_endpoints(
    blocks: &[Block],
    path: &Path,
    config: &Config,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let endpoints: Vec<_> = blocks
        .iter()
        .enumerate()
        .filter_map(|(index, heading)| {
            endpoint_name(heading, &config.labels)?;
            let endpoint = blocks.get(index..=index + children(blocks, index, 2).len())?;
            Some((heading, endpoint))
        })
        .collect();
    for (heading, endpoint) in endpoints {
        diagnostics.extend(validate_path(endpoint, path, config));
        diagnostics.extend(validate_sections(
            endpoint,
            heading.span.start,
            path,
            config,
        ));
        validate_details(endpoint, path, &config.labels, diagnostics);
    }
}

pub fn is_general_components(block: &Block, labels: &Labels) -> bool {
    matches!(block.kind, BlockKind::Heading { level: 2, .. })
        && block.text.trim() == labels.components
}

pub fn is_general_references(block: &Block, level: u8, labels: &Labels) -> bool {
    level == 3
        && matches!(block.kind, BlockKind::Heading { level: 3, .. })
        && block.text.trim() == labels.references
}

fn validate_general_components(
    blocks: &[Block],
    path: &Path,
    config: &Config,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let sections = blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| is_general_components(block, &config.labels));
    for (index, block) in sections {
        let contents = children(blocks, index, 2);
        if !contents.iter().any(|child| {
            matches!(child.kind, BlockKind::Heading { level: 3, .. })
                && !is_general_references(child, 3, &config.labels)
        }) {
            diagnostics.push(issue(
                Rule::Mds007,
                path,
                block,
                format!("Empty section: {}", config.labels.components),
            ));
        }
        diagnostics.extend(validate_components(contents, 3, path, &config.labels));
    }
}

pub fn endpoint_name<'a>(block: &'a Block, labels: &Labels) -> Option<&'a str> {
    if !matches!(block.kind, BlockKind::Heading { level: 2, .. }) {
        return None;
    }
    block
        .text
        .trim()
        .strip_prefix(labels.endpoint_prefix.as_str())
        .map(str::trim)
}

fn validate_path(blocks: &[Block], path: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let paths: Vec<_> = blocks
        .iter()
        .enumerate()
        .take_while(|(_, block)| !matches!(block.kind, BlockKind::Heading { level: 3, .. }))
        .filter(|(_, block)| {
            matches!(block.kind, BlockKind::Paragraph)
                && block
                    .text
                    .trim()
                    .starts_with(config.labels.path_prefix.as_str())
        })
        .collect();
    if paths.is_empty() {
        diagnostics.push(Diagnostic::error(
            Rule::Mds002,
            path,
            blocks.first().map(|block| block.span.start),
            format!("Missing {} METHOD /path", config.labels.path_prefix),
        ));
    }
    for (index, block) in paths {
        let immediately_after_endpoint = index == 1;
        if !immediately_after_endpoint || !valid_path(&block.text, &config.labels) {
            diagnostics.push(issue(
                Rule::Mds002,
                path,
                block,
                format!(
                    "Expected one {} METHOD /path paragraph immediately after the endpoint heading",
                    config.labels.path_prefix
                ),
            ));
        }
    }
    diagnostics
}

fn valid_path(text: &str, labels: &Labels) -> bool {
    let Some(value) = text.trim().strip_prefix(labels.path_prefix.as_str()) else {
        return false;
    };
    let mut parts = value.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();
    matches!(
        method,
        "GET" | "HEAD" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS" | "CONNECT" | "TRACE"
    ) && path.starts_with('/')
        && parts.next().is_none()
        && !value.contains(['\n', '\r'])
}

fn validate_sections(
    blocks: &[Block],
    endpoint: Position,
    path: &Path,
    config: &Config,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = HashSet::new();
    let mut next_rank = 0;
    for (index, block) in blocks.iter().enumerate() {
        if !matches!(block.kind, BlockKind::Heading { level: 3, .. }) {
            continue;
        }
        let name = block.text.trim();
        let Some(rank) = config
            .labels
            .sections()
            .iter()
            .position(|section| *section == name)
        else {
            continue;
        };
        let unique = seen.insert(name);
        let invalid_order = if config.document.enforce_section_order {
            rank < next_rank
        } else {
            !unique
        };
        if invalid_order {
            diagnostics.push(issue(
                Rule::Mds004,
                path,
                block,
                format!(
                    "Sections must be unique and ordered: {}",
                    config.labels.sections().join(", ")
                ),
            ));
        }
        next_rank = next_rank.max(rank + 1);
        let contents = children(blocks, index, 3);
        let populated = match rank {
            0 => contents.iter().any(has_content),
            1..=3 => contents
                .iter()
                .any(|child| matches!(child.kind, BlockKind::Heading { level: 4, .. })),
            _ => true,
        };
        if !populated {
            diagnostics.push(issue(
                Rule::Mds007,
                path,
                block,
                format!("Empty section: {name}"),
            ));
        }
    }
    diagnostics.extend(validate_required_sections(&seen, endpoint, path, config));
    diagnostics
}

fn validate_required_sections(
    seen: &HashSet<&str>,
    endpoint: Position,
    path: &Path,
    config: &Config,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (name, required) in [
        (config.labels.logic.as_str(), config.document.require_logic),
        (
            config.labels.components.as_str(),
            config.document.require_components,
        ),
        (
            config.labels.mappings.as_str(),
            config.document.require_mappings,
        ),
        (
            config.labels.examples.as_str(),
            config.document.require_examples,
        ),
    ] {
        if required && !seen.contains(name) {
            diagnostics.push(Diagnostic::error(
                Rule::Mds003,
                path,
                Some(endpoint),
                format!("Missing required section: {name}"),
            ));
        }
    }
    diagnostics
}

fn has_content(block: &Block) -> bool {
    matches!(
        block.kind,
        BlockKind::Paragraph | BlockKind::Code | BlockKind::Content | BlockKind::Table(_)
    ) && !block.text.trim().is_empty()
}

fn validate_components(
    blocks: &[Block],
    level: u8,
    path: &Path,
    labels: &Labels,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut names = HashSet::new();
    for (index, block) in blocks.iter().enumerate() {
        if !matches!(block.kind, BlockKind::Heading { level: depth, .. } if depth == level)
            || is_general_references(block, level, labels)
        {
            continue;
        }
        if !names.insert(block.text.trim()) {
            diagnostics.push(issue(
                Rule::Mds009,
                path,
                block,
                format!("Duplicate component name: {}", block.text),
            ));
        }
        let contents = children(blocks, index, level);
        let descriptions: Vec<_> = contents
            .iter()
            .enumerate()
            .filter(|(_, child)| {
                matches!(child.kind, BlockKind::Heading { level: depth, .. } if depth == level + 1)
                    && child.text.trim() == labels.description
            })
            .collect();
        if descriptions.is_empty() {
            diagnostics.push(issue(
                Rule::Mds008,
                path,
                block,
                format!(
                    "Component {} is missing H{} {}",
                    block.text,
                    level + 1,
                    labels.description
                ),
            ));
        }
        for (number, (description_index, description)) in descriptions.iter().enumerate() {
            if number > 0
                || !children(contents, *description_index, level + 1)
                    .iter()
                    .any(has_content)
            {
                diagnostics.push(issue(
                    Rule::Mds008,
                    path,
                    description,
                    format!(
                        "Component {} must have one nonempty description",
                        block.text
                    ),
                ));
            }
        }
    }
    diagnostics
}

fn validate_examples(
    blocks: &[Block],
    path: &Path,
    labels: &Labels,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = HashSet::new();
    for (index, block) in blocks.iter().enumerate() {
        if !matches!(block.kind, BlockKind::Heading { level: 4, .. }) {
            continue;
        }
        let name = block.text.trim();
        let known = name == labels.request || name == labels.response;
        let has_code = children(blocks, index, 4)
            .iter()
            .any(|child| matches!(child.kind, BlockKind::Code) && !child.text.trim().is_empty());
        let message = if !known {
            format!(
                "Example {name} must be named {} or {}",
                labels.request, labels.response
            )
        } else if !names.insert(name) {
            format!("Duplicate example: {name}")
        } else if !has_code {
            format!("Example {name} must contain a nonempty fenced code block")
        } else {
            continue;
        };
        diagnostics.push(issue(Rule::Mds017, path, block, message));
    }
}

fn validate_mappings(
    blocks: &[Block],
    path: &Path,
    labels: &Labels,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut inside_mapping = false;
    for (index, block) in blocks.iter().enumerate() {
        if matches!(block.kind, BlockKind::Heading { level: 4, .. }) {
            inside_mapping = true;
            let contents = children(blocks, index, 4);
            let tables: Vec<_> = contents
                .iter()
                .take_while(|child| !matches!(child.kind, BlockKind::Heading { .. }))
                .filter(|child| matches!(child.kind, BlockKind::Table(_)))
                .collect();
            if tables.is_empty() {
                diagnostics.push(issue(
                    Rule::Mds010,
                    path,
                    block,
                    format!(
                        "Mapping {} must contain a table directly under its H4 heading",
                        block.text
                    ),
                ));
            }
            for table in tables {
                validate_table(table, path, labels, diagnostics);
            }
        } else if matches!(block.kind, BlockKind::Table(_)) && !inside_mapping {
            diagnostics.push(issue(
                Rule::Mds010,
                path,
                block,
                "A mapping table must belong to a named H4 mapping",
            ));
        }
    }
}

fn validate_table(block: &Block, path: &Path, labels: &Labels, diagnostics: &mut Vec<Diagnostic>) {
    let BlockKind::Table(rows) = &block.kind else {
        return;
    };
    let headers = rows.first().map(Vec::as_slice).unwrap_or_default();
    let source = headers.iter().position(|name| name.trim() == labels.source);
    let target = headers.iter().position(|name| name.trim() == labels.target);
    let (Some(source), Some(target)) = (source, target) else {
        diagnostics.push(issue(
            Rule::Mds010,
            path,
            block,
            format!(
                "A mapping table must have {} and {} columns",
                labels.source, labels.target
            ),
        ));
        return;
    };
    if rows.len() < 2
        || rows.iter().skip(1).any(|row| {
            [source, target]
                .iter()
                .any(|index| row.get(*index).is_none_or(|value| value.trim().is_empty()))
        })
    {
        diagnostics.push(issue(
            Rule::Mds010,
            path,
            block,
            format!(
                "A mapping table must have data rows with nonempty {} and {} cells",
                labels.source, labels.target
            ),
        ));
    }
}
