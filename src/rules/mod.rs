use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::model::Document;
use crate::outline::Outline;
use std::path::Path;

mod component_scope;
mod components;
mod examples;
mod mappings;
mod path;
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
    unmigrated,
];

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
            crate::references::validate(source.document, &source.outline, source.path, config)
        })
        .collect()
}
