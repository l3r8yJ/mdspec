use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::model::{BlockKind, Document, Position};
use crate::references::{is_local, percent_decode};
use crate::rule::Rule;
use crate::rules::{children, endpoint_name, is_general_components, is_general_references};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    General,
    Endpoint(usize),
}

struct Component<'a> {
    path: &'a Path,
    file: PathBuf,
    anchor: String,
    name: &'a str,
    position: Position,
    extent: Range<usize>,
    scope: Scope,
}

type Key = (PathBuf, String);

struct Usage<'a> {
    path: &'a Path,
    source: PathBuf,
    file: PathBuf,
    anchor: String,
    position: Position,
    scope: Option<Scope>,
}

pub fn validate(documents: &[(PathBuf, Document)], config: &Config) -> Vec<Diagnostic> {
    let components: Vec<_> = documents
        .iter()
        .flat_map(|(path, document)| declared(path, document, config))
        .collect();
    let usages: Vec<_> = documents
        .iter()
        .flat_map(|(path, document)| used(path, document, config))
        .collect();
    let mut index: HashMap<Key, &Component> = HashMap::new();
    for component in &components {
        index
            .entry((component.file.clone(), component.anchor.clone()))
            .or_insert(component);
    }
    let referenced = referenced_keys(&usages, &index);
    components
        .iter()
        .filter(|component| component.scope == Scope::General)
        .filter_map(|component| unreferenced(component, &referenced))
        .chain(
            usages
                .iter()
                .filter_map(|usage| misplaced(usage, &index, config)),
        )
        .collect()
}

fn regions(document: &Document, config: &Config) -> Vec<(Range<usize>, Option<Scope>)> {
    let headings: Vec<_> = document
        .blocks
        .iter()
        .filter(|block| matches!(block.kind, BlockKind::Heading { level: 2, .. }))
        .collect();
    headings
        .iter()
        .enumerate()
        .map(|(number, heading)| {
            let end = headings
                .get(number + 1)
                .map_or(usize::MAX, |next| next.span.start.offset);
            let scope = if is_general_components(heading, &config.labels) {
                Some(Scope::General)
            } else {
                endpoint_name(heading, &config.labels)
                    .map(|_| Scope::Endpoint(heading.span.start.offset))
            };
            (heading.span.start.offset..end, scope)
        })
        .collect()
}

fn scope_at(regions: &[(Range<usize>, Option<Scope>)], offset: usize) -> Option<Scope> {
    regions
        .iter()
        .find(|(range, _)| range.contains(&offset))
        .and_then(|(_, scope)| *scope)
}

fn declared<'a>(path: &'a Path, document: &'a Document, config: &Config) -> Vec<Component<'a>> {
    let file = canonical(path);
    let regions = regions(document, config);
    let mut components = Vec::new();
    for (index, block) in document.blocks.iter().enumerate() {
        let (level, scope) = match block.kind {
            BlockKind::Heading { level: 2, .. } if is_general_components(block, &config.labels) => {
                (3, Scope::General)
            }
            BlockKind::Heading { level: 3, .. }
                if block.text.trim() == config.labels.components =>
            {
                match scope_at(&regions, block.span.start.offset) {
                    Some(Scope::Endpoint(endpoint)) => (4, Scope::Endpoint(endpoint)),
                    _ => continue,
                }
            }
            _ => continue,
        };
        let section = children(&document.blocks, index, level - 1);
        for (offset, heading) in section.iter().enumerate() {
            if !matches!(heading.kind, BlockKind::Heading { level: depth, .. } if depth == level)
                || is_general_references(heading, level, &config.labels)
            {
                continue;
            }
            let BlockKind::Heading { anchor, .. } = &heading.kind else {
                continue;
            };
            let end = children(section, offset, level)
                .last()
                .map_or(heading.span.end.offset, |last| last.span.end.offset);
            components.push(Component {
                path,
                file: file.clone(),
                anchor: anchor.clone(),
                name: heading.text.trim(),
                position: heading.span.start,
                extent: heading.span.start.offset..end,
                scope,
            });
        }
    }
    components
}

fn used<'a>(path: &'a Path, document: &Document, config: &Config) -> Vec<Usage<'a>> {
    let source = canonical(path);
    let regions = regions(document, config);
    document
        .links
        .iter()
        .filter_map(|link| {
            let (file, anchor) = local_target(link.url.as_deref()?, path)?;
            Some(Usage {
                path,
                source: source.clone(),
                file,
                anchor,
                position: link.position,
                scope: scope_at(&regions, link.position.offset),
            })
        })
        .collect()
}

fn local_target(url: &str, from: &Path) -> Option<(PathBuf, String)> {
    if !is_local(url) {
        return None;
    }
    let (resource, fragment) = url.split_once('#')?;
    let resource = resource
        .split_once('?')
        .map_or(resource, |(resource, _)| resource);
    let resource = percent_decode(resource)?;
    let target = if resource.is_empty() {
        from.to_path_buf()
    } else {
        from.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(resource)
    };
    Some((fs::canonicalize(target).ok()?, percent_decode(fragment)?))
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn referenced_keys(usages: &[Usage], index: &HashMap<Key, &Component>) -> HashSet<Key> {
    usages
        .iter()
        .filter(|usage| {
            index
                .get(&(usage.file.clone(), usage.anchor.clone()))
                .is_some_and(|component| {
                    !(usage.source == component.file
                        && component.extent.contains(&usage.position.offset))
                })
        })
        .map(|usage| (usage.file.clone(), usage.anchor.clone()))
        .collect()
}

fn unreferenced(component: &Component, referenced: &HashSet<Key>) -> Option<Diagnostic> {
    let referenced = referenced.contains(&(component.file.clone(), component.anchor.clone()));
    (!referenced).then(|| {
        Diagnostic::error(
            Rule::Mds018,
            component.path,
            Some(component.position),
            format!(
                "General component {} is not referenced by any endpoint or component",
                component.name
            ),
        )
    })
}

fn misplaced(
    usage: &Usage,
    index: &HashMap<Key, &Component>,
    config: &Config,
) -> Option<Diagnostic> {
    let component = index.get(&(usage.file.clone(), usage.anchor.clone()))?;
    let inside = usage.source == component.file && usage.scope == Some(component.scope);
    (component.scope != Scope::General && !inside).then(|| {
        let mut diagnostic = Diagnostic::error(
            Rule::Mds019,
            usage.path,
            Some(usage.position),
            format!(
                "Internal component {} is referenced outside its endpoint",
                component.name
            ),
        );
        diagnostic.help = Some(format!(
            "Move {} to the H2 {} section or remove the link",
            component.name, config.labels.components
        ));
        diagnostic
    })
}
