use crate::config::{Config, Labels};
use crate::diagnostics::Diagnostic;
use crate::model::BlockKind;
use crate::outline::{Item, Section, SectionKind};
use crate::rule::Rule;
use crate::rules::{Source, endpoint_section_diagnostics};
use std::collections::HashSet;
use std::path::Path;

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    endpoint_section_diagnostics(sources, SectionKind::Examples, |section, path| {
        examples(section, path, &config.labels)
    })
}

fn examples(section: &Section, path: &Path, labels: &Labels) -> Vec<Diagnostic> {
    let mut names = HashSet::new();
    section
        .items
        .iter()
        .filter_map(|item| {
            problem(item, &mut names, labels).map(|message| {
                Diagnostic::error(Rule::Mds017, path, Some(item.heading.span.start), message)
            })
        })
        .collect()
}

fn problem<'a>(item: &'a Item, names: &mut HashSet<&'a str>, labels: &Labels) -> Option<String> {
    let name = item.heading.text.trim();
    let known = name == labels.request || name == labels.response;
    let has_code = item
        .body
        .iter()
        .any(|child| matches!(child.kind, BlockKind::Code) && !child.text.trim().is_empty());
    if !known {
        Some(format!(
            "Example {name} must be named {} or {}",
            labels.request, labels.response
        ))
    } else if !names.insert(name) {
        Some(format!("Duplicate example: {name}"))
    } else if !has_code {
        Some(format!(
            "Example {name} must contain a nonempty fenced code block"
        ))
    } else {
        None
    }
}
