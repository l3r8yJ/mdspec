use crate::config::{Config, Labels};
use crate::diagnostics::Diagnostic;
use crate::model::{Block, BlockKind};
use crate::outline::{Item, SectionKind, children, level};
use crate::rule::Rule;
use crate::rules::Source;
use std::collections::HashSet;
use std::path::Path;

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    let labels = &config.labels;
    sources
        .iter()
        .flat_map(|source| {
            let endpoint_items = source
                .outline
                .endpoints()
                .flat_map(|endpoint| &endpoint.sections)
                .filter(|section| section.kind == SectionKind::Components)
                .flat_map(|section| {
                    described(
                        &section.items,
                        level::SECTION_ITEM_DESCRIPTION,
                        source.path,
                        labels,
                    )
                });
            let general_items = source.outline.general().flat_map(|general| {
                described(
                    &general.items,
                    level::GENERAL_ITEM_DESCRIPTION,
                    source.path,
                    labels,
                )
            });
            endpoint_items.chain(general_items)
        })
        .collect()
}

fn described(
    items: &[Item],
    description_level: u8,
    path: &Path,
    labels: &Labels,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut names = HashSet::new();
    for item in items {
        let block = item.heading;
        if !names.insert(block.text.trim()) {
            diagnostics.push(Diagnostic::error(
                Rule::Mds009,
                path,
                Some(block.span.start),
                format!("Duplicate component name: {}", block.text),
            ));
        }
        let descriptions: Vec<_> = item
            .body
            .iter()
            .enumerate()
            .filter(|(_, child)| {
                matches!(child.kind, BlockKind::Heading { level: depth, .. } if depth == description_level)
                    && child.text.trim() == labels.description
            })
            .collect();
        if descriptions.is_empty() {
            diagnostics.push(Diagnostic::error(
                Rule::Mds008,
                path,
                Some(block.span.start),
                format!(
                    "Component {} is missing H{} {}",
                    block.text, description_level, labels.description
                ),
            ));
        }
        for (number, (description_index, description)) in descriptions.iter().enumerate() {
            if number > 0
                || !children(item.body, *description_index, description_level)
                    .iter()
                    .any(Block::has_content)
            {
                diagnostics.push(Diagnostic::error(
                    Rule::Mds008,
                    path,
                    Some(description.span.start),
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
