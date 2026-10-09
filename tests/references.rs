#![cfg(test)]

use asserting::prelude::*;
use serde_json::Value;
use std::process::Command;

fn check(file: &str, strict: bool) -> (Option<i32>, Value) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mdspec"));
    command.args(["check", file, "--format", "json"]);
    if strict {
        command.arg("--strict");
    }
    let output = command.output().expect("built binary starts");
    let json = serde_json::from_slice(&output.stdout).expect("stdout contains JSON diagnostics");
    (output.status.code(), json)
}

#[test]
fn resolves_normalized_references_and_unicode_duplicate_anchors() {
    let (code, json) = check("tests/fixtures/references/valid.md", true);
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len()
    )
    .is_equal_to(0);
}

#[test]
fn reports_all_reference_failures_and_strict_unused_definitions() {
    let (code, json) = check("tests/fixtures/references/invalid.md", true);
    assert_that!(code).is_equal_to(Some(1));
    let rules: Vec<&str> = json["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .filter_map(|item| item["rule"].as_str())
        .collect();
    for rule in ["MDS011", "MDS012", "MDS013", "MDS014", "MDS016"] {
        assert_that!(rules.contains(&rule)).is_equal_to(true);
    }
}

#[test]
fn resolves_encoded_fragments_queries_and_same_file_links() {
    let (code, json) = check("tests/fixtures/references/encoded.md", true);
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len()
    )
    .is_equal_to(0);
}

#[test]
fn rejects_nonterminal_and_misplaced_definitions() {
    let (code, json) = check("tests/fixtures/references/placement.md", false);
    assert_that!(code).is_equal_to(Some(1));
    let diagnostics = json["diagnostics"].as_array().expect("diagnostics array");
    assert_that!(diagnostics.iter().any(|item| item["rule"] == "MDS015")).is_equal_to(true);
    assert_that!(
        diagnostics
            .iter()
            .all(|item| item["line"].is_number() && item["column"].is_number())
    )
    .is_equal_to(true);
}

#[test]
fn strict_mode_rejects_unused_definitions() {
    let (normal_code, normal) = check("tests/fixtures/references/unused.md", false);
    let (strict_code, strict) = check("tests/fixtures/references/unused.md", true);
    assert_that!(normal_code).is_equal_to(Some(0));
    assert_that!(strict_code).is_equal_to(Some(1));
    assert_that!(
        normal["diagnostics"]
            .as_array()
            .expect("normal diagnostics")
            .len()
    )
    .is_equal_to(0);
    let diagnostics = strict["diagnostics"]
        .as_array()
        .expect("strict diagnostics");
    assert_that!(diagnostics.len()).is_equal_to(1);
    assert_that!(diagnostics[0]["rule"].as_str()).is_equal_to(Some("MDS016"));
    assert_that!(diagnostics[0]["severity"].as_str()).is_equal_to(Some("error"));
}
