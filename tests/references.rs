#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::{Workspace, check_fixture, rule};

fn document(content: &str) -> String {
    format!("{}\n{content}\n", include_str!("fixtures/minimal.md"))
}

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

#[test]
fn accepts_an_inline_image_of_an_existing_file_without_a_references_section() {
    let workspace = Workspace::new();
    workspace.write("diagram.svg", b"<svg/>");
    let (status, report) = workspace.check_document(&document("![Схема](diagram.svg)"), "", true);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(common::rules_with_lines(&report)).is_empty();
}

#[test]
fn reports_an_inline_image_of_a_missing_file() {
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&document("![Схема](diagram.svg)"), "", true);
    assert_that!(common::rules_with_lines(&report)).is_equal_to(vec![common::rule("MDS012", 9)]);
}

#[test]
fn counts_a_reference_style_image_as_using_its_definition() {
    let workspace = Workspace::new();
    workspace.write("diagram.svg", b"<svg/>");
    let (status, report) = workspace.check_document(
        &document("![Схема][diagram]\n\n### Ссылки\n\n[diagram]: diagram.svg"),
        "",
        true,
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(common::rules_with_lines(&report)).is_empty();
}

#[test]
fn ignores_a_late_title_after_the_references_section() {
    let document = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\n[x][a].\n\n## Ссылки\n\n[a]: #эндпоинт-а\n\n# Поздний\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(document, "", true)).is_equal_to(vec![rule("MDS005", 13)]);
}
