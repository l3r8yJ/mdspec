use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::model::Document;
use crate::outline::{Outline, Section, SectionKind};
use std::path::Path;

mod component_scope;
mod components;
mod examples;
mod mappings;
mod path;
mod references;
mod sections;
mod structure;

pub struct Source<'a> {
    pub path: &'a Path,
    pub document: &'a Document,
    pub outline: Outline<'a>,
}

type Check = fn(&[Source<'_>], &Config) -> Vec<Diagnostic>;

const CHECKS: &[Check] = &[
    structure::check,
    sections::check,
    path::check,
    components::check,
    component_scope::check,
    mappings::check,
    examples::check,
    references::check,
];

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    CHECKS
        .iter()
        .flat_map(|check| check(sources, config))
        .collect()
}

fn section_diagnostics(
    sources: &[Source],
    kind: SectionKind,
    check: impl Fn(&Section, &Path) -> Vec<Diagnostic>,
) -> Vec<Diagnostic> {
    sources
        .iter()
        .flat_map(|source| {
            source
                .outline
                .endpoints()
                .flat_map(|endpoint| &endpoint.sections)
                .filter(|section| section.kind == kind)
                .flat_map(|section| check(section, source.path))
        })
        .collect()
}
