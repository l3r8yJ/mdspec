#![cfg(test)]

use asserting::prelude::*;
use std::process::Command;

fn check(source: &str, config: &str, target: &[u8]) -> (Option<i32>, serde_json::Value) {
    let directory = tempfile::tempdir().expect("temporary directory exists");
    let document = directory.path().join("document.md");
    let configuration = directory.path().join("mdspec.toml");
    std::fs::write(&document, source).expect("document written");
    std::fs::write(&configuration, config).expect("configuration written");
    std::fs::write(directory.path().join("target.md"), target).expect("target written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(document)
        .args(["--format", "json", "--config"])
        .arg(configuration)
        .output()
        .expect("binary runs");
    let report = serde_json::from_slice(&output.stdout).expect("JSON report");
    (output.status.code(), report)
}

fn document(content: &str) -> String {
    format!("{}\n{content}\n", include_str!("fixtures/minimal.md"))
}

#[test]
fn requires_references_section_for_an_unused_definition() {
    let (status, report) = check(&document("[target]: target.md"), "", b"# Target");
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn requires_references_section_for_an_external_inline_link() {
    let (status, report) = check(&document("[external](https://example.com)"), "", b"");
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS015"));
}

#[test]
fn validates_files_when_anchor_validation_is_disabled() {
    let (status, report) = check(
        &document("[missing][file]\n\n### Ссылки\n\n[file]: absent.md"),
        "[references]\nvalidate_anchors = false\n",
        b"",
    );
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS012"));
}

#[test]
fn validates_anchors_when_file_validation_is_disabled() {
    let (status, report) = check(
        &document("[missing][file]\n\n### Ссылки\n\n[file]: target.md#missing"),
        "[references]\nvalidate_local_paths = false\n",
        b"# Target",
    );
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS013"));
}

#[test]
fn reports_unreadable_markdown_when_resolving_an_anchor() {
    let (status, report) = check(
        &document("[target][file]\n\n### Ссылки\n\n[file]: target.md#target"),
        "",
        &[0xff],
    );
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS012"));
}

#[test]
fn checks_local_names_containing_a_colon() {
    let (status, report) = check(
        &document("[target][file]\n\n### Ссылки\n\n[file]: archive/target:old.md"),
        "",
        b"",
    );
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS012"));
}

#[test]
fn rejects_a_reference_definition_inside_a_quote() {
    let (status, report) = check(
        &document("[target][file]\n\n### Ссылки\n\n> [file]: target.md"),
        "",
        b"# Target",
    );
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS015"));
}

#[test]
fn reports_definition_position_when_content_follows_it() {
    let (status, report) = check(
        &document("[target][file]\n\n### Ссылки\n\n[file]: target.md\n\nTrailing content"),
        "",
        b"# Target",
    );
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS015"));
    assert_that!(report["diagnostics"][0]["line"].as_u64()).is_equal_to(Some(13));
}
