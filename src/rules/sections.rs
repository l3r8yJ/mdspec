use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::model::Block;
use crate::outline::{Endpoint, Outline, Part, Section, SectionKind};
use crate::rule::Rule;
use crate::rules::Source;
use std::collections::HashSet;
use std::path::Path;

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    sources
        .iter()
        .flat_map(|source| {
            let (outline, path) = (&source.outline, source.path);
            let mut diagnostics = placement(outline, path, config);
            for endpoint in outline.endpoints() {
                diagnostics.extend(order(endpoint, path, config));
                diagnostics.extend(
                    endpoint
                        .sections
                        .iter()
                        .filter_map(|section| empty(section, path)),
                );
                diagnostics.extend(required(endpoint, path, config));
            }
            diagnostics.extend(empty_general(outline, path, config));
            diagnostics
        })
        .collect()
}

fn order(endpoint: &Endpoint, path: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut seen = HashSet::new();
    let mut next_rank = 0;
    let mut diagnostics = Vec::new();
    for section in &endpoint.sections {
        let Some(rank) = section.kind.rank() else {
            continue;
        };
        let unique = seen.insert(section.heading.text.trim());
        let invalid_order = if config.document.enforce_section_order {
            rank < next_rank
        } else {
            !unique
        };
        if invalid_order {
            diagnostics.push(Diagnostic::error(
                Rule::Mds004,
                path,
                Some(section.heading.span.start),
                format!(
                    "Sections must be unique and ordered: {}",
                    config.labels.sections().join(", ")
                ),
            ));
        }
        next_rank = next_rank.max(rank + 1);
    }
    diagnostics
}

fn empty(section: &Section, path: &Path) -> Option<Diagnostic> {
    let populated = match section.kind {
        SectionKind::Logic => section.body.iter().any(Block::has_content),
        SectionKind::Components | SectionKind::Mappings | SectionKind::Examples => {
            !section.items.is_empty()
        }
        SectionKind::References | SectionKind::Unknown => true,
    };
    (!populated).then(|| {
        Diagnostic::error(
            Rule::Mds007,
            path,
            Some(section.heading.span.start),
            format!("Empty section: {}", section.heading.text.trim()),
        )
    })
}

fn required(endpoint: &Endpoint, path: &Path, config: &Config) -> Vec<Diagnostic> {
    [
        (
            SectionKind::Logic,
            config.document.require_logic,
            &config.labels.logic,
        ),
        (
            SectionKind::Components,
            config.document.require_components,
            &config.labels.components,
        ),
        (
            SectionKind::Mappings,
            config.document.require_mappings,
            &config.labels.mappings,
        ),
        (
            SectionKind::Examples,
            config.document.require_examples,
            &config.labels.examples,
        ),
    ]
    .into_iter()
    .filter(|(kind, required, _)| {
        *required
            && !endpoint
                .sections
                .iter()
                .any(|section| section.kind == *kind)
    })
    .map(|(_, _, name)| {
        Diagnostic::error(
            Rule::Mds003,
            path,
            Some(endpoint.heading.span.start),
            format!("Missing required section: {name}"),
        )
    })
    .collect()
}

fn empty_general(outline: &Outline, path: &Path, config: &Config) -> Vec<Diagnostic> {
    outline
        .general()
        .filter(|general| general.items.is_empty())
        .map(|general| {
            Diagnostic::error(
                Rule::Mds007,
                path,
                Some(general.heading.span.start),
                format!("Empty section: {}", config.labels.components),
            )
        })
        .collect()
}

fn placement(outline: &Outline, path: &Path, config: &Config) -> Vec<Diagnostic> {
    outline
        .parts
        .iter()
        .enumerate()
        .filter_map(|(index, part)| match part {
            Part::General(general) => Some((index, general)),
            Part::Endpoint(_) | Part::Other(_) => None,
        })
        .enumerate()
        .filter(|(number, (index, _))| {
            *number > 0
                || outline
                    .parts
                    .iter()
                    .skip(*index)
                    .any(|part| matches!(part, Part::Endpoint(_)))
        })
        .map(|(_, (_, general))| {
            Diagnostic::error(
                Rule::Mds004,
                path,
                Some(general.heading.span.start),
                format!(
                    "The H2 {} section must appear once, after every endpoint",
                    config.labels.components
                ),
            )
        })
        .collect()
}
