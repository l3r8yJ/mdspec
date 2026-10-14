#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::{Workspace, check_strict, config_check, label_error};

#[test]
fn accepts_custom_english_labels_throughout_the_document() {
    let (code, report) = check_strict(
        "tests/fixtures/languages/english.md",
        "tests/fixtures/languages/english.toml",
    );
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn accepts_unicode_labels_without_a_builtin_language_pack() {
    let (code, report) = check_strict(
        "tests/fixtures/languages/japanese.md",
        "tests/fixtures/languages/japanese.toml",
    );
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn rejects_ambiguous_section_labels_as_configuration_errors() {
    let (code, report) = config_check("[labels]\nlogic = 'Same'\ncomponents = 'Same'\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS900"));
}

#[test]
fn rejects_blank_labels_before_validation() {
    let (code, report) = config_check("[labels]\nendpoint_prefix = ''\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS900"));
    assert_that!(report["diagnostics"][0]["message"].as_str())
        .is_equal_to(Some("labels.endpoint_prefix must not be empty"));
}

#[test]
fn names_the_key_when_a_label_has_surrounding_whitespace() {
    let (code, message) = label_error("[labels]\nlogic = ' Logic '\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(message).is_equal_to("labels.logic must not have surrounding whitespace");
}

#[test]
fn names_the_key_when_a_label_spans_multiple_lines() {
    let (code, message) = label_error("[labels]\nlogic = '''a\nb'''\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(message).is_equal_to("labels.logic must not span multiple lines");
}

#[test]
fn names_both_keys_when_section_labels_are_duplicated() {
    let (code, message) = label_error("[labels]\nlogic = 'Same'\ncomponents = 'Same'\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(message).is_equal_to("labels.logic and labels.components must differ");
}

#[test]
fn names_both_keys_when_non_adjacent_section_labels_are_duplicated() {
    let (code, message) = label_error("[labels]\nlogic = 'Same'\nreferences = 'Same'\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(message).is_equal_to("labels.logic and labels.references must differ");
}

#[test]
fn names_both_keys_when_the_source_and_target_labels_are_equal() {
    let (code, message) = label_error("[labels]\nsource = 'X'\ntarget = 'X'\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(message).is_equal_to("labels.source and labels.target must differ");
}

#[test]
fn names_both_keys_when_the_request_and_response_labels_are_equal() {
    let (code, message) = label_error("[labels]\nrequest = 'X'\nresponse = 'X'\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(message).is_equal_to("labels.request and labels.response must differ");
}

#[test]
fn reports_every_invalid_label_in_one_diagnostic() {
    let (code, message) =
        label_error("[labels]\nendpoint_prefix = ' '\nrequest = 'X'\nresponse = 'X'\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(message).is_equal_to(
        "labels.endpoint_prefix must not have surrounding whitespace; \
         labels.request and labels.response must differ",
    );
}

#[test]
fn enforces_configured_labels_instead_of_accepting_any_heading() {
    let workspace = Workspace::new();
    let document = workspace.path().join("invalid.md");
    let source = include_str!("fixtures/languages/english.md")
        .replace("Route:", "Path:")
        .replace("##### Details", "##### Описание")
        .replace("| From |", "| Source |")
        .replace("### Resources", "### Ссылки");
    std::fs::write(&document, source).expect("document written");
    let (code, report) = check_strict(
        document.to_str().expect("UTF8 temporary path"),
        "tests/fixtures/languages/english.toml",
    );
    let rules = common::rules(&report);
    assert_that!(code).is_equal_to(Some(1));
    assert_that!(rules).contains_all_of(["MDS002", "MDS008", "MDS010", "MDS015"]);
}

#[test]
fn rejects_identical_mapping_column_labels() {
    let (code, report) = config_check("[labels]\nsource = 'Value'\ntarget = 'Value'\n");
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS900"));
}

#[test]
fn preserves_default_labels_when_only_one_label_is_overridden() {
    let (code, report) = config_check("[labels]\nsource = 'Origin'\n");
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn rejects_the_removed_multiple_endpoints_key() {
    let (code, message) = common::label_error("[document]\nmultiple_endpoints = true\n");
    assert_that!((code, message.contains("multiple_endpoints"))).is_equal_to((Some(2), true));
}
