use crate::config::{Config, Labels};
use crate::diagnostics::Diagnostic;
use crate::model::{Block, BlockKind};
use crate::outline::{Item, Section, SectionKind, level};
use crate::rule::Rule;
use crate::rules::{Source, endpoint_section_diagnostics};
use std::path::Path;

const HEADER_AND_ONE_ROW: usize = 2;

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    endpoint_section_diagnostics(sources, SectionKind::Mappings, |section, path| {
        mapping(section, path, &config.labels)
    })
}

fn mapping(section: &Section, path: &Path, labels: &Labels) -> Vec<Diagnostic> {
    let lead = section
        .body
        .iter()
        .take_while(|block| !is_item_heading(block))
        .filter(|block| matches!(block.kind, BlockKind::Table(_)))
        .map(|block| {
            error(
                path,
                block,
                "A mapping table must belong to a named H4 mapping",
            )
        });
    let items = section
        .items
        .iter()
        .flat_map(|item| item_diagnostics(item, path, labels));
    lead.chain(items).collect()
}

const fn is_item_heading(block: &Block) -> bool {
    matches!(
        block.kind,
        BlockKind::Heading {
            level: level::SECTION_ITEM,
            ..
        }
    )
}

fn item_diagnostics(item: &Item, path: &Path, labels: &Labels) -> Vec<Diagnostic> {
    let tables: Vec<_> = item
        .body
        .iter()
        .take_while(|block| !matches!(block.kind, BlockKind::Heading { .. }))
        .filter_map(|block| match &block.kind {
            BlockKind::Table(rows) => Some((block, rows.as_slice())),
            _ => None,
        })
        .collect();
    if tables.is_empty() {
        return vec![error(
            path,
            item.heading,
            format!(
                "Mapping {} must contain a table directly under its H4 heading",
                item.heading.text
            ),
        )];
    }
    tables
        .into_iter()
        .filter_map(|(block, rows)| table_diagnostic(block, rows, path, labels))
        .collect()
}

fn table_diagnostic(
    block: &Block,
    rows: &[Vec<String>],
    path: &Path,
    labels: &Labels,
) -> Option<Diagnostic> {
    let headers = rows.first().map(Vec::as_slice).unwrap_or_default();
    let source = headers.iter().position(|name| name.trim() == labels.source);
    let target = headers.iter().position(|name| name.trim() == labels.target);
    let (Some(source), Some(target)) = (source, target) else {
        return Some(error(
            path,
            block,
            format!(
                "A mapping table must have {} and {} columns",
                labels.source, labels.target
            ),
        ));
    };
    let incomplete = rows.len() < HEADER_AND_ONE_ROW
        || rows.iter().skip(1).any(|row| {
            [source, target]
                .iter()
                .any(|index| row.get(*index).is_none_or(|value| value.trim().is_empty()))
        });
    incomplete.then(|| {
        error(
            path,
            block,
            format!(
                "A mapping table must have data rows with nonempty {} and {} cells",
                labels.source, labels.target
            ),
        )
    })
}

fn error(path: &Path, block: &Block, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(Rule::Mds010, path, Some(block.span.start), message)
}
