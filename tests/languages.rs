use asserting::prelude::*;
use serde_json::Value;
use std::process::Command;

fn check(document: &str, config: &str) -> (Option<i32>, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args([
            "check", document, "--config", config, "--strict", "--format", "json",
        ])
        .output()
        .expect("binary starts");
    let report = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    (output.status.code(), report)
}

#[test]
fn accepts_custom_english_labels_throughout_the_document() {
    let (code, report) = check(
        "tests/fixtures/languages/english.md",
        "tests/fixtures/languages/english.toml",
    );
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn accepts_unicode_labels_without_a_builtin_language_pack() {
    let (code, report) = check(
        "tests/fixtures/languages/japanese.md",
        "tests/fixtures/languages/japanese.toml",
    );
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn rejects_ambiguous_section_labels_as_configuration_errors() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let config = directory.path().join("invalid.toml");
    std::fs::write(&config, "[labels]\nlogic = 'Same'\ncomponents = 'Same'\n")
        .expect("config written");
    let (code, report) = check(
        "tests/fixtures/minimal.md",
        config.to_str().expect("UTF8 temporary path"),
    );
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS900"));
}

#[test]
fn rejects_blank_labels_before_validation() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let config = directory.path().join("invalid.toml");
    std::fs::write(&config, "[labels]\nendpoint_prefix = ''\n").expect("config written");
    let (code, report) = check(
        "tests/fixtures/minimal.md",
        config.to_str().expect("UTF8 temporary path"),
    );
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS900"));
}

#[test]
fn enforces_configured_labels_instead_of_accepting_any_heading() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let document = directory.path().join("invalid.md");
    let source = include_str!("fixtures/languages/english.md")
        .replace("Route:", "Path:")
        .replace("##### Details", "##### Описание")
        .replace("| From |", "| Source |")
        .replace("### Resources", "### Ссылки");
    std::fs::write(&document, source).expect("document written");
    let (code, report) = check(
        document.to_str().expect("UTF8 temporary path"),
        "tests/fixtures/languages/english.toml",
    );
    let rules: Vec<_> = report["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .filter_map(|item| item["rule"].as_str())
        .collect();
    assert_that!(code).is_equal_to(Some(1));
    assert_that!(rules).contains_all_of(["MDS002", "MDS008", "MDS010", "MDS015"]);
}

#[test]
fn rejects_identical_mapping_column_labels() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let config = directory.path().join("invalid.toml");
    std::fs::write(&config, "[labels]\nsource = 'Value'\ntarget = 'Value'\n")
        .expect("config written");
    let (code, report) = check(
        "tests/fixtures/minimal.md",
        config.to_str().expect("UTF8 temporary path"),
    );
    assert_that!(code).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS900"));
}

#[test]
fn preserves_default_labels_when_only_one_label_is_overridden() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let config = directory.path().join("partial.toml");
    std::fs::write(&config, "[labels]\nsource = 'Origin'\n").expect("config written");
    let (code, report) = check(
        "tests/fixtures/minimal.md",
        config.to_str().expect("UTF8 temporary path"),
    );
    assert_that!(code).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}
