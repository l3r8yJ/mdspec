use crate::config::{Config, Labels};
use crate::diagnostics::Diagnostic;
use crate::model::{Block, BlockKind, Document};
use crate::outline::{General, Item, Outline, Section, SectionKind, children, level};
use crate::rule::Rule;
use std::collections::HashSet;
use std::path::Path;

mod path;
mod sections;
mod structure;

pub struct Source<'a> {
    pub path: &'a Path,
    pub document: &'a Document,
    pub outline: Outline<'a>,
}

type Check = fn(&[Source<'_>], &Config) -> Vec<Diagnostic>;

const CHECKS: &[Check] = &[structure::check, sections::check, path::check, unmigrated];

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    CHECKS
        .iter()
        .flat_map(|check| check(sources, config))
        .collect()
}

fn unmigrated(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    sources
        .iter()
        .flat_map(|source| {
            let mut diagnostics = endpoint_rules(&source.outline, source.path, config);
            diagnostics.extend(crate::references::validate(
                source.document,
                &source.outline,
                source.path,
                config,
            ));
            diagnostics
        })
        .chain(crate::components::validate(sources, config))
        .collect()
}

fn endpoint_rules(outline: &Outline, path: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for endpoint in outline.endpoints() {
        validate_details(&endpoint.sections, path, &config.labels, &mut diagnostics);
    }
    for general in outline.general() {
        diagnostics.extend(validate_general_components(general, path, &config.labels));
    }
    diagnostics
}

fn validate_details(
    sections: &[Section],
    path: &Path,
    labels: &Labels,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for section in sections {
        match section.kind {
            SectionKind::Components => {
                diagnostics.extend(validate_components(
                    &section.items,
                    level::SECTION_ITEM,
                    path,
                    labels,
                ));
            }
            SectionKind::Mappings => validate_mappings(section.body, path, labels, diagnostics),
            SectionKind::Examples => validate_examples(section.body, path, labels, diagnostics),
            SectionKind::Logic | SectionKind::References | SectionKind::Unknown => {}
        }
    }
}

fn validate_general_components(general: &General, path: &Path, labels: &Labels) -> Vec<Diagnostic> {
    validate_components(&general.items, level::GENERAL_ITEM, path, labels)
}

fn validate_components(items: &[Item], level: u8, path: &Path, labels: &Labels) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut names = HashSet::new();
    for item in items {
        let block = item.heading;
        if !names.insert(block.text.trim()) {
            diagnostics.push(issue(
                Rule::Mds009,
                path,
                block,
                format!("Duplicate component name: {}", block.text),
            ));
        }
        let descriptions: Vec<_> = item
            .body
            .iter()
            .enumerate()
            .filter(|(_, child)| {
                matches!(child.kind, BlockKind::Heading { level: depth, .. } if depth == level::below(level))
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
                    level::below(level),
                    labels.description
                ),
            ));
        }
        for (number, (description_index, description)) in descriptions.iter().enumerate() {
            if number > 0
                || !children(item.body, *description_index, level::below(level))
                    .iter()
                    .any(Block::has_content)
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

fn issue(rule: Rule, path: &Path, block: &Block, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(rule, path, Some(block.span.start), message)
}

fn validate_examples(
    blocks: &[Block],
    path: &Path,
    labels: &Labels,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = HashSet::new();
    for (index, block) in blocks.iter().enumerate() {
        if !matches!(
            block.kind,
            BlockKind::Heading {
                level: level::SECTION_ITEM,
                ..
            }
        ) {
            continue;
        }
        let name = block.text.trim();
        let known = name == labels.request || name == labels.response;
        let has_code = children(blocks, index, level::SECTION_ITEM)
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

const HEADER_AND_ONE_ROW: usize = 2;

fn validate_mappings(
    blocks: &[Block],
    path: &Path,
    labels: &Labels,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut inside_mapping = false;
    for (index, block) in blocks.iter().enumerate() {
        if matches!(
            block.kind,
            BlockKind::Heading {
                level: level::SECTION_ITEM,
                ..
            }
        ) {
            inside_mapping = true;
            let contents = children(blocks, index, level::SECTION_ITEM);
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
    if rows.len() < HEADER_AND_ONE_ROW
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
