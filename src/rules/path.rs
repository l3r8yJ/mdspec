use crate::config::{Config, Labels};
use crate::diagnostics::Diagnostic;
use crate::model::BlockKind;
use crate::outline::Endpoint;
use crate::rule::Rule;
use crate::rules::Source;
use std::path::Path;

pub fn check(sources: &[Source], config: &Config) -> Vec<Diagnostic> {
    sources
        .iter()
        .flat_map(|source| {
            source
                .outline
                .endpoints()
                .flat_map(|endpoint| validate_path(endpoint, source.path, config))
        })
        .collect()
}

fn validate_path(endpoint: &Endpoint, path: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let paths: Vec<_> = endpoint
        .intro
        .iter()
        .enumerate()
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
            Some(endpoint.heading.span.start),
            format!("Missing {} METHOD /path", config.labels.path_prefix),
        ));
    }
    for (index, block) in paths {
        let immediately_after_endpoint = index == 0;
        if !immediately_after_endpoint || !valid_path(&block.text, &config.labels) {
            diagnostics.push(Diagnostic::error(
                Rule::Mds002,
                path,
                Some(block.span.start),
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
