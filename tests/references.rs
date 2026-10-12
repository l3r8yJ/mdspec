#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::check_fixture;

#[test]
fn resolves_normalized_references_and_unicode_duplicate_anchors() {
    let (code, json) = check_fixture("tests/fixtures/references/valid.md", true);
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
    let (code, json) = check_fixture("tests/fixtures/references/invalid.md", true);
    assert_that!(code).is_equal_to(Some(1));
    let rules = common::rules(&json);
    for rule in ["MDS011", "MDS012", "MDS013", "MDS014", "MDS016"] {
        assert_that!(rules.contains(&rule)).is_equal_to(true);
    }
}

#[test]
fn resolves_encoded_fragments_queries_and_same_file_links() {
    let (code, json) = check_fixture("tests/fixtures/references/encoded.md", true);
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
fn resolves_suffixed_anchors_for_repeated_headings() {
    let (code, json) = check_fixture("tests/fixtures/references/duplicates.md", true);
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
    let (code, json) = check_fixture("tests/fixtures/references/placement.md", false);
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
fn warns_about_unused_definitions_without_strict() {
    let (code, json) = check_fixture("tests/fixtures/references/unused.md", false);
    let diagnostics = json["diagnostics"].as_array().expect("diagnostics array");
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(json["error_count"].as_u64()).is_equal_to(Some(0));
    assert_that!(json["warning_count"].as_u64()).is_equal_to(Some(1));
    assert_that!(diagnostics.len()).is_equal_to(1);
    assert_that!(diagnostics[0]["rule"].as_str()).is_equal_to(Some("MDS016"));
    assert_that!(diagnostics[0]["severity"].as_str()).is_equal_to(Some("warning"));
}

#[test]
fn strict_promotes_unused_definition_warnings_to_errors() {
    let (code, json) = check_fixture("tests/fixtures/references/unused.md", true);
    let diagnostics = json["diagnostics"].as_array().expect("diagnostics array");
    assert_that!(code).is_equal_to(Some(1));
    assert_that!(json["error_count"].as_u64()).is_equal_to(Some(1));
    assert_that!(json["warning_count"].as_u64()).is_equal_to(Some(0));
    assert_that!(diagnostics.len()).is_equal_to(1);
    assert_that!(diagnostics[0]["rule"].as_str()).is_equal_to(Some("MDS016"));
    assert_that!(diagnostics[0]["severity"].as_str()).is_equal_to(Some("error"));
}

#[test]
fn configuration_promotes_unused_definition_warnings_to_errors() {
    let (code, json) = common::report(&[
        "check",
        "tests/fixtures/references/unused.md",
        "--format",
        "json",
        "--config",
        "tests/fixtures/references/warnings_as_errors.toml",
    ]);
    let diagnostics = json["diagnostics"].as_array().expect("diagnostics array");
    assert_that!(code).is_equal_to(Some(1));
    assert_that!(json["error_count"].as_u64()).is_equal_to(Some(1));
    assert_that!(json["warning_count"].as_u64()).is_equal_to(Some(0));
    assert_that!(diagnostics.len()).is_equal_to(1);
    assert_that!(diagnostics[0]["rule"].as_str()).is_equal_to(Some("MDS016"));
    assert_that!(diagnostics[0]["severity"].as_str()).is_equal_to(Some("error"));
}
