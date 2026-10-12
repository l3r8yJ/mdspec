#![cfg(test)]

use asserting::prelude::*;
use std::process::Command;

const ALLOWED: &str = "[references]\ntoc_allowed = true\n";
const ENDPOINT: &str = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст.\n";

fn check(marker: &str, references: &str, config: &str) -> Vec<String> {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let document_path = directory.path().join("service.md");
    let config_path = directory.path().join("mdspec.toml");
    std::fs::write(
        &document_path,
        format!("{marker}\n\n{ENDPOINT}{references}"),
    )
    .expect("document written");
    std::fs::write(&config_path, config).expect("configuration written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(document_path)
        .args(["--strict", "--format", "json", "--config"])
        .arg(config_path)
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    report["diagnostics"]
        .as_array()
        .expect("diagnostics is an array")
        .iter()
        .map(|diagnostic| diagnostic["rule"].as_str().expect("rule string").to_owned())
        .collect()
}

#[test]
fn accepts_a_standalone_toc_marker_when_allowed() {
    assert_that!(check("[TOC]", "", ALLOWED)).is_empty();
}

#[test]
fn accepts_the_gitlab_toc_marker_when_allowed() {
    assert_that!(check("[[_TOC_]]", "", ALLOWED)).is_empty();
}

#[test]
fn reports_a_toc_reference_inside_a_sentence_even_when_allowed() {
    assert_that!(check("Смотри [TOC] ниже.", "", ALLOWED))
        .contains_exactly(["MDS011".to_owned(), "MDS015".to_owned()]);
}

#[test]
fn reports_a_toc_marker_inside_a_blockquote_even_when_allowed() {
    assert_that!(check("> [TOC]", "", ALLOWED))
        .contains_exactly(["MDS011".to_owned(), "MDS015".to_owned()]);
}

#[test]
fn reports_a_standalone_toc_marker_by_default() {
    assert_that!(check("[TOC]", "", ""))
        .contains_exactly(["MDS011".to_owned(), "MDS015".to_owned()]);
}

#[test]
fn keeps_a_defined_toc_reference_as_a_link_when_allowed() {
    let references = "\n### Ссылки\n\n[TOC]: #эндпоинт-а\n";
    assert_that!(check("[TOC]", references, ALLOWED)).is_empty();
}
