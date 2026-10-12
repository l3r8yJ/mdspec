#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::Workspace;

#[test]
fn accepts_a_document_saved_with_a_byte_order_mark() {
    let fixture = std::fs::read("tests/fixtures/bom/minimal.md").expect("fixture readable");
    let output = common::output(&["check", "tests/fixtures/bom/minimal.md", "--format", "json"]);
    let (status, report) = common::parse(&output);
    assert_that!(fixture.starts_with(&[0xEF, 0xBB, 0xBF])).is_equal_to(true);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn reports_a_link_at_column_six_of_line_one_in_a_byte_order_mark_document() {
    let workspace = Workspace::new();
    workspace.write("endpoint.md", "\u{FEFF}## A [x](target.md#nope)\n");
    workspace.write("target.md", "# Target\n");
    let (_, report) = workspace.check(&["endpoint.md"]);
    let diagnostic = report["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .find(|diagnostic| diagnostic["rule"] == "MDS013")
        .expect("MDS013 reported");
    assert_that!(diagnostic["line"].as_u64()).is_equal_to(Some(1));
    assert_that!(diagnostic["column"].as_u64()).is_equal_to(Some(6));
}

#[test]
fn resolves_an_anchor_in_the_first_heading_of_a_byte_order_mark_target() {
    let document = "## Эндпоинт: Ссылки\n\nPath: `GET /links`\n\n### Логика работы\n\n1. Проверить [цель][target].\n\n### Ссылки\n\n[target]: target.md#первый-заголовок\n";
    let workspace = Workspace::new();
    workspace.write("endpoint.md", document);
    workspace.write("target.md", "\u{FEFF}# Первый заголовок\n");
    let (code, report) = workspace.check(&["endpoint.md"]);
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}
