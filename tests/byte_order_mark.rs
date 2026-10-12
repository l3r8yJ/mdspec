#![cfg(test)]

use asserting::prelude::*;
use serde_json::Value;
use std::process::Command;

fn check(document: &str, target: &str) -> (Option<i32>, Value) {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let document_path = directory.path().join("endpoint.md");
    std::fs::write(&document_path, document).expect("document written");
    std::fs::write(directory.path().join("target.md"), target).expect("target written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args(["check", "--format", "json"])
        .arg(&document_path)
        .output()
        .expect("the built binary starts");
    let report = serde_json::from_slice(&output.stdout).expect("stdout contains JSON diagnostics");
    (output.status.code(), report)
}

#[test]
fn accepts_a_document_saved_with_a_byte_order_mark() {
    let fixture = std::fs::read("tests/fixtures/bom/minimal.md").expect("fixture readable");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args(["check", "tests/fixtures/bom/minimal.md", "--format", "json"])
        .output()
        .expect("the built binary starts");
    let report: Value =
        serde_json::from_slice(&output.stdout).expect("stdout contains JSON diagnostics");
    assert_that!(fixture.starts_with(&[0xEF, 0xBB, 0xBF])).is_equal_to(true);
    assert_that!(output.status.code()).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn reports_a_link_at_column_six_of_line_one_in_a_byte_order_mark_document() {
    let (_, report) = check("\u{FEFF}## A [x](target.md#nope)\n", "# Target\n");
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
    let (code, report) = check(document, "\u{FEFF}# Первый заголовок\n");
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}
