use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::model::Block;
use crate::outline::{Outline, Part, SectionKind};
use crate::rule::Rule;
use crate::rules::Source;
use std::path::Path;

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    sources
        .iter()
        .flat_map(|source| {
            let (outline, path) = (&source.outline, source.path);
            let mut diagnostics = endpoint_headings(outline, path, config);
            diagnostics.extend(placement(outline, path, config));
            diagnostics.extend(nesting(outline, path, config));
            diagnostics.extend(unknown_sections(outline, path, config));
            diagnostics
        })
        .collect()
}

fn endpoint_headings(outline: &Outline, path: &Path, config: &Config) -> Vec<Diagnostic> {
    let prefix = &config.labels.endpoint_prefix;
    let missing = config.document.require_endpoint
        && outline
            .parts
            .iter()
            .all(|part| matches!(part, Part::Other(_)));
    let missing = missing.then(|| {
        Diagnostic::error(
            Rule::Mds001,
            path,
            None,
            format!("Missing H2 heading: {prefix} name"),
        )
    });
    let unnamed = outline
        .endpoints()
        .filter(|endpoint| endpoint.name.is_empty())
        .map(|endpoint| {
            Diagnostic::error(
                Rule::Mds001,
                path,
                Some(endpoint.heading.span.start),
                format!("Expected H2 heading: {prefix} name"),
            )
        });
    missing.into_iter().chain(unnamed).collect()
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

fn nesting(outline: &Outline, path: &Path, config: &Config) -> Vec<Diagnostic> {
    let labels = &config.labels;
    let message = format!(
        "Invalid heading nesting: H2 endpoint, H3 section, H4 component/mapping, H5 {}; H2 {}, H3 component, H4 {}",
        labels.description, labels.components, labels.description
    );
    let preamble: &[&Block] = if config.document.require_endpoint {
        &outline.preamble
    } else {
        &[]
    };
    outline
        .stray
        .iter()
        .chain(preamble)
        .map(|heading| {
            Diagnostic::error(
                Rule::Mds005,
                path,
                Some(heading.span.start),
                message.clone(),
            )
        })
        .chain(outline.nested.iter().map(|heading| {
            Diagnostic::error(
                Rule::Mds005,
                path,
                Some(heading.position),
                "Structural headings must not occur inside lists or blockquotes",
            )
        }))
        .collect()
}

fn unknown_sections(outline: &Outline, path: &Path, config: &Config) -> Vec<Diagnostic> {
    if config.document.allow_unknown_sections {
        return Vec::new();
    }
    outline
        .endpoints()
        .flat_map(|endpoint| &endpoint.sections)
        .filter(|section| section.kind == SectionKind::Unknown)
        .map(|section| {
            Diagnostic::error(
                Rule::Mds006,
                path,
                Some(section.heading.span.start),
                format!("Unknown section: {}", section.heading.text.trim()),
            )
        })
        .collect()
}
