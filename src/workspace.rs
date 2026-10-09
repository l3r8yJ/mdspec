use crate::diagnostics::Diagnostic;
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

pub fn discover(path: &Path) -> (Vec<PathBuf>, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return (
                Vec::new(),
                vec![Diagnostic::error(
                    "MDS901",
                    path,
                    None,
                    format!("Could not open input path: {error}"),
                )],
            );
        }
    };
    if metadata.is_file() {
        return if is_markdown(path) {
            (vec![path.to_path_buf()], diagnostics)
        } else {
            (
                Vec::new(),
                vec![Diagnostic::error(
                    "MDS902",
                    path,
                    None,
                    "Expected a file with the .md extension",
                )],
            )
        };
    }
    if !metadata.is_dir() {
        return (
            Vec::new(),
            vec![Diagnostic::error(
                "MDS902",
                path,
                None,
                "Expected a regular file or directory",
            )],
        );
    }
    let mut files = Vec::new();
    for entry in WalkBuilder::new(path)
        .require_git(false)
        .follow_links(false)
        .build()
    {
        match entry {
            Ok(entry)
                if entry.file_type().is_some_and(|kind| kind.is_file())
                    && is_markdown(entry.path()) =>
            {
                files.push(entry.into_path())
            }
            Ok(_) => (),
            Err(error) => diagnostics.push(Diagnostic::error(
                "MDS901",
                path,
                None,
                format!("Could not traverse directory: {error}"),
            )),
        }
    }
    files.sort();
    if files.is_empty() && diagnostics.is_empty() {
        diagnostics.push(Diagnostic::error(
            "MDS902",
            path,
            None,
            "No Markdown files found",
        ));
    }
    (files, diagnostics)
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}
