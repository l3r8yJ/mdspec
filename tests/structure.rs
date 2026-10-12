#![cfg(test)]

mod common;

use asserting::prelude::*;

#[test]
fn accepts_complete_document_with_unicode_and_code_blocks() {
    assert_that!(common::exit_and_rule_names(
        "tests/fixtures/structure/complete.md"
    ))
    .is_equal_to((0, vec![]));
}

#[test]
fn accepts_document_without_optional_sections() {
    assert_that!(common::exit_and_rule_names(
        "tests/fixtures/structure/minimal.md"
    ))
    .is_equal_to((0, vec![]));
}

#[test]
fn reports_only_the_missing_endpoint_in_an_empty_document() {
    assert_that!(common::exit_and_rule_names(
        "tests/fixtures/structure/empty.md"
    ))
    .is_equal_to((1, vec!["MDS001".to_owned()]));
}

#[test]
fn rejects_invalid_method_and_section_order() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/order.md");
    assert_that!(rules).contains_all_of(["MDS002".to_owned(), "MDS004".to_owned()]);
}

#[test]
fn reports_component_and_mapping_errors() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/invalid.md");
    assert_that!(rules).contains_all_of([
        "MDS008".to_owned(),
        "MDS009".to_owned(),
        "MDS010".to_owned(),
        "MDS005".to_owned(),
    ]);
}

#[test]
fn does_not_use_a_code_block_as_endpoint_path() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/code-path.md");
    assert_that!(rules).contains("MDS002".to_owned());
}

#[test]
fn reports_a_missing_path_in_nonempty_document() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/missing-path.md");
    assert_that!(rules).contains("MDS002".to_owned());
}

#[test]
fn rejects_empty_required_mapping_cells() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/empty-cell.md");
    assert_that!(rules).contains("MDS010".to_owned());
}

#[test]
fn does_not_use_comments_as_component_description() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/comment-description.md");
    assert_that!(rules).contains("MDS008".to_owned());
}

#[test]
fn rejects_a_table_outside_a_named_mapping() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/orphan-table.md");
    assert_that!(rules).contains("MDS010".to_owned());
}

#[test]
fn permits_shared_sections_when_endpoint_is_optional() {
    let output = common::output(&[
        "check",
        "tests/fixtures/structure/shared.md",
        "--config",
        "tests/fixtures/structure/shared.toml",
        "--format",
        "json",
    ]);
    assert_that!(output.status.code()).is_equal_to(Some(0));
}

#[test]
fn permits_component_prose_starting_with_path() {
    assert_that!(common::exit_and_rule_names(
        "tests/fixtures/structure/component-path-prose.md"
    ))
    .is_equal_to((0, vec![]));
}

#[test]
fn does_not_accept_logic_prose_as_missing_endpoint_path() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/late-path.md");
    assert_that!(rules).contains("MDS002".to_owned());
}
