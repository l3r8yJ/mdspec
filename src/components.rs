use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::model::{BlockKind, Document, LinkKind, Position};
use crate::outline::{Item, Outline, Part, SectionKind};
use crate::references::{is_local, percent_decode};
use crate::rule::Rule;

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

pub fn validate(documents: &[(&Path, &Document, Outline)], config: &Config) -> Vec<Diagnostic> {
    let components: Vec<_> = documents
        .iter()
        .flat_map(|(path, _, outline)| declared(path, outline))
        .collect();
    let usages: Vec<_> = documents
        .iter()
        .flat_map(|(path, document, outline)| used(path, document, outline))
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

fn scope_at(outline: &Outline, offset: usize) -> Option<Scope> {
    match outline.part_at(offset)? {
        (index, Part::Endpoint(_)) => Some(Scope::Endpoint(index)),
        (_, Part::General(_)) => Some(Scope::General),
        (_, Part::Other(_)) => None,
    }
}

fn declared<'a>(path: &'a Path, outline: &'a Outline) -> Vec<Component<'a>> {
    let file = canonical(path);
    outline
        .parts
        .iter()
        .enumerate()
        .flat_map(|(index, part)| scoped_items(index, part))
        .filter_map(|(scope, item)| {
            let BlockKind::Heading { anchor, .. } = &item.heading.kind else {
                return None;
            };
            let end = item
                .body
                .last()
                .map_or(item.heading.span.end.offset, |last| last.span.end.offset);
            Some(Component {
                path,
                file: file.clone(),
                anchor: anchor.clone(),
                name: item.heading.text.trim(),
                position: item.heading.span.start,
                extent: item.heading.span.start.offset..end,
                scope,
            })
        })
        .collect()
}

fn scoped_items<'o, 'a>(index: usize, part: &'o Part<'a>) -> Vec<(Scope, &'o Item<'a>)> {
    match part {
        Part::Endpoint(endpoint) => endpoint
            .sections
            .iter()
            .chain(&endpoint.detached)
            .filter(|section| section.kind == SectionKind::Components)
            .flat_map(|section| &section.items)
            .map(|item| (Scope::Endpoint(index), item))
            .collect(),
        Part::General(general) => general
            .items
            .iter()
            .map(|item| (Scope::General, item))
            .collect(),
        Part::Other(_) => Vec::new(),
    }
}

fn used<'a>(path: &'a Path, document: &Document, outline: &Outline) -> Vec<Usage<'a>> {
    let source = canonical(path);
    document
        .links
        .iter()
        .filter(|link| link.kind == LinkKind::Link)
        .filter_map(|link| {
            let (file, anchor) = local_target(link.url.as_deref()?, path)?;
            Some(Usage {
                path,
                source: source.clone(),
                file,
                anchor,
                position: link.position,
                scope: scope_at(outline, link.position.offset),
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
