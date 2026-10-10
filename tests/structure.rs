#![cfg(test)]
use asserting::prelude::*;
use serde_json::Value;
use std::process::Command;

fn report(fixture: &str) -> (i32, Vec<String>) {
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args([
            "check",
            &format!("tests/fixtures/structure/{fixture}.md"),
            "--format",
            "json",
        ])
        .output()
        .expect("CLI starts");
    let report: Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    let rules = report["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .map(|value| value["rule"].as_str().expect("rule string").to_owned())
        .collect();
    (output.status.code().expect("exit code"), rules)
}

#[test]
fn accepts_complete_document_with_unicode_and_code_blocks() {
    assert_that!(report("complete")).is_equal_to((0, vec![]));
}

#[test]
fn accepts_document_without_optional_sections() {
    assert_that!(report("minimal")).is_equal_to((0, vec![]));
}

#[test]
fn reports_only_the_missing_endpoint_in_an_empty_document() {
    assert_that!(report("empty")).is_equal_to((1, vec!["MDS001".to_owned()]));
}

#[test]
fn rejects_invalid_method_and_section_order() {
    let (_, rules) = report("order");
    assert_that!(rules).contains_all_of(["MDS002".to_owned(), "MDS004".to_owned()]);
}

#[test]
fn reports_component_and_mapping_errors() {
    let (_, rules) = report("invalid");
    assert_that!(rules).contains_all_of([
        "MDS008".to_owned(),
        "MDS009".to_owned(),
        "MDS010".to_owned(),
        "MDS005".to_owned(),
    ]);
}

#[test]
fn does_not_use_a_code_block_as_endpoint_path() {
    let (_, rules) = report("code-path");
    assert_that!(rules).contains("MDS002".to_owned());
}

#[test]
fn reports_a_missing_path_in_nonempty_document() {
    let (_, rules) = report("missing-path");
    assert_that!(rules).contains("MDS002".to_owned());
}

#[test]
fn rejects_empty_required_mapping_cells() {
    let (_, rules) = report("empty-cell");
    assert_that!(rules).contains("MDS010".to_owned());
}

#[test]
fn does_not_use_comments_as_component_description() {
    let (_, rules) = report("comment-description");
    assert_that!(rules).contains("MDS008".to_owned());
}

#[test]
fn rejects_a_table_outside_a_named_mapping() {
    let (_, rules) = report("orphan-table");
    assert_that!(rules).contains("MDS010".to_owned());
}

#[test]
fn permits_shared_sections_when_endpoint_is_optional() {
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args([
            "check",
            "tests/fixtures/structure/shared.md",
            "--config",
            "tests/fixtures/structure/shared.toml",
            "--format",
            "json",
        ])
        .output()
        .expect("CLI starts");
    assert_that!(output.status.code()).is_equal_to(Some(0));
}

#[test]
fn permits_component_prose_starting_with_path() {
    assert_that!(report("component-path-prose")).is_equal_to((0, vec![]));
}

#[test]
fn does_not_accept_logic_prose_as_missing_endpoint_path() {
    let (_, rules) = report("late-path");
    assert_that!(rules).contains("MDS002".to_owned());
}
