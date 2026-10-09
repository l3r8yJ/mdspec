use crate::config::{Config, Labels};
use crate::diagnostics::Diagnostic;
use crate::model::{Block, BlockKind, Document};
use std::collections::HashSet;
use std::path::Path;

pub fn validate(document: &Document, path: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    validate_endpoint(document, path, config, &mut diagnostics);
    validate_sections(document, path, config, &mut diagnostics);
    validate_nesting(document, path, config, &mut diagnostics);
    for (index, block) in document.blocks.iter().enumerate() {
        if !matches!(block.kind, BlockKind::Heading(3)) {
            continue;
        }
        let children = children(&document.blocks, index, 3);
        if block.text.trim() == config.labels.components {
            validate_components(children, path, &config.labels, &mut diagnostics);
        } else if block.text.trim() == config.labels.mappings {
            validate_mappings(children, path, &config.labels, &mut diagnostics);
        }
    }
    diagnostics
}

fn children(blocks: &[Block], index: usize, depth: u8) -> &[Block] {
    let rest = blocks.get(index + 1..).unwrap_or_default();
    let length = rest
        .iter()
        .position(|block| matches!(block.kind, BlockKind::Heading(level) if level <= depth))
        .unwrap_or(rest.len());
    rest.get(..length).unwrap_or_default()
}

fn issue(rule: &'static str, path: &Path, block: &Block, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(rule, path, Some(block.position), message)
}

fn validate_endpoint(
    document: &Document,
    path: &Path,
    config: &Config,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let headings: Vec<_> = document
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| matches!(block.kind, BlockKind::Heading(2)))
        .collect();
    if headings.is_empty() && config.document.require_endpoint {
        diagnostics.push(Diagnostic::error(
            "MDS001",
            path,
            None,
            format!("Missing H2 heading: {} name", config.labels.endpoint_prefix),
        ));
    }
    for (number, (_, heading)) in headings.iter().enumerate() {
        if number > 0
            || heading
                .text
                .trim()
                .strip_prefix(config.labels.endpoint_prefix.as_str())
                .is_none_or(|name| name.trim().is_empty())
        {
            diagnostics.push(issue(
                "MDS001",
                path,
                heading,
                format!(
                    "Expected one H2 heading: {} name",
                    config.labels.endpoint_prefix
                ),
            ));
        }
    }
    let paths: Vec<_> = document
        .blocks
        .iter()
        .enumerate()
        .take_while(|(_, block)| !matches!(block.kind, BlockKind::Heading(3)))
        .filter(|(_, block)| {
            matches!(block.kind, BlockKind::Paragraph)
                && block
                    .text
                    .trim()
                    .starts_with(config.labels.path_prefix.as_str())
        })
        .collect();
    if paths.is_empty() && (config.document.require_endpoint || !headings.is_empty()) {
        diagnostics.push(Diagnostic::error(
            "MDS002",
            path,
            headings.first().map(|(_, block)| block.position),
            format!("Missing {} METHOD /path", config.labels.path_prefix),
        ));
    }
    for (number, (index, block)) in paths.iter().enumerate() {
        let immediately_after_endpoint = headings
            .first()
            .is_some_and(|(heading_index, _)| *index == heading_index + 1);
        if number > 0 || !immediately_after_endpoint || !valid_path(&block.text, &config.labels) {
            diagnostics.push(issue(
                "MDS002",
                path,
                block,
                format!(
                    "Expected one {} METHOD /path paragraph immediately after the endpoint heading",
                    config.labels.path_prefix
                ),
            ));
        }
    }
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
    document: &Document,
    path: &Path,
    config: &Config,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = HashSet::new();
    let mut last_rank = None;
    for (index, block) in document.blocks.iter().enumerate() {
        if !matches!(block.kind, BlockKind::Heading(3)) {
            continue;
        }
        let name = block.text.trim();
        let Some(rank) = config
            .labels
            .sections()
            .iter()
            .position(|section| *section == name)
        else {
            if !config.document.allow_unknown_sections {
                diagnostics.push(issue(
                    "MDS006",
                    path,
                    block,
                    format!("Unknown section: {name}"),
                ));
            }
            continue;
        };
        if !seen.insert(name)
            || (config.document.enforce_section_order && last_rank.is_some_and(|last| rank < last))
        {
            diagnostics.push(issue(
                "MDS004",
                path,
                block,
                format!(
                    "Sections must be unique and ordered: {}",
                    config.labels.sections().join(", ")
                ),
            ));
        }
        last_rank = Some(last_rank.map_or(rank, |last: usize| last.max(rank)));
        let contents = children(&document.blocks, index, 3);
        let populated = match rank {
            0 => contents.iter().any(has_content),
            1 | 2 => contents
                .iter()
                .any(|child| matches!(child.kind, BlockKind::Heading(4))),
            _ => true,
        };
        if !populated {
            diagnostics.push(issue(
                "MDS007",
                path,
                block,
                format!("Empty section: {name}"),
            ));
        }
    }
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
    ] {
        if required && !seen.contains(name) {
            diagnostics.push(Diagnostic::error(
                "MDS003",
                path,
                None,
                format!("Missing required section: {name}"),
            ));
        }
    }
}

fn has_content(block: &Block) -> bool {
    matches!(
        block.kind,
        BlockKind::Paragraph | BlockKind::Content | BlockKind::Table(_)
    ) && !block.text.trim().is_empty()
}

fn validate_nesting(
    document: &Document,
    path: &Path,
    config: &Config,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut section = "";
    let mut component = false;
    let mut endpoint = !config.document.require_endpoint;
    for block in &document.blocks {
        let BlockKind::Heading(depth) = block.kind else {
            continue;
        };
        let valid = match depth {
            2 => {
                endpoint = true;
                section = "";
                component = false;
                true
            }
            3 => {
                section = block.text.trim();
                component = false;
                endpoint
            }
            4 => {
                component = section == config.labels.components;
                (component || section == config.labels.mappings) && !block.text.trim().is_empty()
            }
            5 => component && block.text.trim() == config.labels.description,
            _ => false,
        };
        if !valid {
            diagnostics.push(issue(
                "MDS005",
                path,
                block,
                format!(
                    "Invalid heading nesting: H2 endpoint, H3 section, H4 component/mapping, H5 {}",
                    config.labels.description
                ),
            ));
        }
    }
    diagnostics.extend(document.nested_headings.iter().map(|position| {
        Diagnostic::error(
            "MDS005",
            path,
            Some(*position),
            "Structural headings must not occur inside lists or blockquotes",
        )
    }));
}

fn validate_components(
    blocks: &[Block],
    path: &Path,
    labels: &Labels,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = HashSet::new();
    for (index, block) in blocks.iter().enumerate() {
        if !matches!(block.kind, BlockKind::Heading(4)) {
            continue;
        }
        if !names.insert(block.text.trim()) {
            diagnostics.push(issue(
                "MDS009",
                path,
                block,
                format!("Duplicate component name: {}", block.text),
            ));
        }
        let contents = children(blocks, index, 4);
        let descriptions: Vec<_> = contents
            .iter()
            .enumerate()
            .filter(|(_, child)| {
                matches!(child.kind, BlockKind::Heading(5))
                    && child.text.trim() == labels.description
            })
            .collect();
        if descriptions.is_empty() {
            diagnostics.push(issue(
                "MDS008",
                path,
                block,
                format!(
                    "Component {} is missing H5 {}",
                    block.text, labels.description
                ),
            ));
        }
        for (number, (description_index, description)) in descriptions.iter().enumerate() {
            if number > 0
                || !children(contents, *description_index, 5)
                    .iter()
                    .any(has_content)
            {
                diagnostics.push(issue(
                    "MDS008",
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
}

fn validate_mappings(
    blocks: &[Block],
    path: &Path,
    labels: &Labels,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut inside_mapping = false;
    for (index, block) in blocks.iter().enumerate() {
        if matches!(block.kind, BlockKind::Heading(4)) {
            inside_mapping = true;
            let contents = children(blocks, index, 4);
            let tables: Vec<_> = contents
                .iter()
                .take_while(|child| !matches!(child.kind, BlockKind::Heading(_)))
                .filter(|child| matches!(child.kind, BlockKind::Table(_)))
                .collect();
            if tables.is_empty() {
                diagnostics.push(issue(
                    "MDS010",
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
                "MDS010",
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
            "MDS010",
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
            "MDS010",
            path,
            block,
            format!(
                "A mapping table must have data rows with nonempty {} and {} cells",
                labels.source, labels.target
            ),
        ));
    }
}
