mod pattern;

use crate::diagnostics::Diagnostic;
use ignore::WalkBuilder;
use pattern::{Pattern, has_magic, normalize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

struct Selection {
    input: PathBuf,
    pattern: Option<Pattern>,
    matched: bool,
}

struct Scan {
    display_root: PathBuf,
    selections: Vec<Selection>,
}

#[derive(Default)]
struct Discovery {
    files: BTreeMap<PathBuf, PathBuf>,
    diagnostics: Vec<Diagnostic>,
    scans: BTreeMap<PathBuf, Scan>,
}

pub fn discover(inputs: &[PathBuf], exclude: &[String]) -> (Vec<PathBuf>, Vec<Diagnostic>) {
    let mut discovery = Discovery::default();
    let exclusions = discovery.exclusions(exclude);
    for input in inputs {
        discovery.select(input, &exclusions);
    }
    let scans = std::mem::take(&mut discovery.scans);
    for (root, scan) in scans {
        discovery.scan(&root, scan, &exclusions);
    }
    if discovery.files.is_empty() && discovery.diagnostics.is_empty() {
        discovery.issue(
            "MDS902",
            Path::new("."),
            "No Markdown files remain after exclusions",
        );
    }
    let mut files: Vec<_> = discovery.files.into_values().collect();
    files.sort();
    (files, discovery.diagnostics)
}

impl Discovery {
    fn exclusions(&mut self, exclude: &[String]) -> Vec<Pattern> {
        exclude
            .iter()
            .filter_map(|text| match Pattern::compile(Path::new(text)) {
                Ok(pattern) => Some(pattern),
                Err(error) => {
                    self.diagnostics.push(*error);
                    None
                }
            })
            .collect()
    }

    fn select(&mut self, input: &Path, exclusions: &[Pattern]) {
        match fs::metadata(input) {
            Ok(metadata) if metadata.is_file() => self.file(input, exclusions),
            Ok(metadata) if metadata.is_dir() => self.add_scan(input, None),
            Ok(_) => self.issue("MDS902", input, "Expected a regular file or directory"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && has_magic(input) => {
                match Pattern::compile(input) {
                    Ok(pattern) => self.add_scan(input, Some(pattern)),
                    Err(error) => self.diagnostics.push(*error),
                }
            }
            Err(error) => self.issue(
                "MDS901",
                input,
                format!("Could not open input path: {error}"),
            ),
        }
    }

    fn file(&mut self, input: &Path, exclusions: &[Pattern]) {
        match normalize(input) {
            Ok(path) if excluded(&path, exclusions) => (),
            Ok(_) if !is_markdown(input) => {
                self.issue("MDS902", input, "Expected a file with the .md extension");
            }
            Ok(_) => self.insert(input),
            Err(error) => self.issue("MDS901", input, error.to_string()),
        }
    }

    fn add_scan(&mut self, input: &Path, pattern: Option<Pattern>) {
        let display_root = pattern.as_ref().map_or_else(
            || input.to_path_buf(),
            |pattern| pattern.display_root.clone(),
        );
        let root = pattern.as_ref().map_or_else(
            || fs::canonicalize(input),
            |pattern| Ok(pattern.root.clone()),
        );
        match root {
            Ok(root) => self
                .scans
                .entry(root)
                .or_insert_with(|| Scan {
                    display_root,
                    selections: Vec::new(),
                })
                .selections
                .push(Selection {
                    input: input.to_path_buf(),
                    pattern,
                    matched: false,
                }),
            Err(error) => self.issue("MDS901", input, error.to_string()),
        }
    }

    fn scan(&mut self, root: &Path, mut scan: Scan, exclusions: &[Pattern]) {
        let before = self.diagnostics.len();
        let walker = WalkBuilder::new(root)
            .require_git(false)
            .follow_links(false)
            .build();
        for entry in walker {
            match entry {
                Ok(entry)
                    if entry.file_type().is_some_and(|kind| kind.is_file())
                        && is_markdown(entry.path()) =>
                {
                    let matched = scan
                        .selections
                        .iter_mut()
                        .fold(false, |matched, selection| {
                            let selected = selection
                                .pattern
                                .as_ref()
                                .is_none_or(|pattern| pattern.matches(entry.path()));
                            selection.matched |= selected;
                            matched | selected
                        });
                    if matched && !excluded(entry.path(), exclusions) {
                        match entry.path().strip_prefix(root) {
                            Ok(relative) => self.insert(&scan.display_root.join(relative)),
                            Err(error) => self.issue("MDS901", entry.path(), error.to_string()),
                        }
                    }
                }
                Ok(_) => (),
                Err(error) => self.issue(
                    "MDS901",
                    root,
                    format!("Could not traverse directory: {error}"),
                ),
            }
        }
        if self.diagnostics.len() == before {
            for selection in scan
                .selections
                .into_iter()
                .filter(|selection| !selection.matched)
            {
                self.issue(
                    "MDS902",
                    &selection.input,
                    "No Markdown files matched this input",
                );
            }
        }
    }

    fn insert(&mut self, path: &Path) {
        match fs::canonicalize(path) {
            Ok(identity) => {
                self.files
                    .entry(identity)
                    .or_insert_with(|| path.to_path_buf());
            }
            Err(error) => self.issue("MDS901", path, error.to_string()),
        }
    }

    fn issue(&mut self, rule: &'static str, path: &Path, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(rule, path, None, message));
    }
}

fn excluded(path: &Path, patterns: &[Pattern]) -> bool {
    path.ancestors()
        .any(|ancestor| patterns.iter().any(|pattern| pattern.matches(ancestor)))
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}
