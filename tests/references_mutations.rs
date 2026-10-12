#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::Workspace;

fn document(content: &str) -> String {
    format!("{}\n{content}\n", include_str!("fixtures/minimal.md"))
}

#[test]
fn requires_references_section_for_an_unused_definition() {
    let workspace = Workspace::new();
    workspace.write("target.md", b"# Target");
    let (status, report) = workspace.check_default(&document("[target]: target.md"));
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn requires_references_section_for_an_external_inline_link() {
    let workspace = Workspace::new();
    workspace.write("target.md", b"");
    let (status, report) = workspace.check_default(&document("[external](https://example.com)"));
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS015"));
}

#[test]
fn validates_files_when_anchor_validation_is_disabled() {
    let workspace = Workspace::new();
    workspace.write("target.md", b"");
    let (status, report) = workspace.check_document(
        &document("[missing][file]\n\n### Ссылки\n\n[file]: absent.md"),
        "[references]\nvalidate_anchors = false\n",
        false,
    );
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS012"));
}

#[test]
fn validates_anchors_when_file_validation_is_disabled() {
    let workspace = Workspace::new();
    workspace.write("target.md", b"# Target");
    let (status, report) = workspace.check_document(
        &document("[missing][file]\n\n### Ссылки\n\n[file]: target.md#missing"),
        "[references]\nvalidate_local_paths = false\n",
        false,
    );
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS013"));
}

#[test]
fn reports_unreadable_markdown_when_resolving_an_anchor() {
    let workspace = Workspace::new();
    workspace.write("target.md", [0xff]);
    let (status, report) = workspace.check_default(&document(
        "[target][file]\n\n### Ссылки\n\n[file]: target.md#target",
    ));
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS012"));
}

#[test]
fn checks_local_names_containing_a_colon() {
    let workspace = Workspace::new();
    workspace.write("target.md", b"");
    let (status, report) = workspace.check_default(&document(
        "[target][file]\n\n### Ссылки\n\n[file]: archive/target:old.md",
    ));
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS012"));
}

#[test]
fn rejects_a_reference_definition_inside_a_quote() {
    let workspace = Workspace::new();
    workspace.write("target.md", b"# Target");
    let (status, report) = workspace.check_default(&document(
        "[target][file]\n\n### Ссылки\n\n> [file]: target.md",
    ));
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS015"));
}

#[test]
fn reports_definition_position_when_content_follows_it() {
    let workspace = Workspace::new();
    workspace.write("target.md", b"# Target");
    let (status, report) = workspace.check_default(&document(
        "[target][file]\n\n### Ссылки\n\n[file]: target.md\n\nTrailing content",
    ));
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS015"));
    assert_that!(report["diagnostics"][0]["line"].as_u64()).is_equal_to(Some(13));
}
