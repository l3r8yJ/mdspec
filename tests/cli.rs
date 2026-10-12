#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::Workspace;

#[test]
fn rejects_empty_document_with_machine_readable_diagnostics() {
    let (status, result) =
        common::report(&["check", "tests/fixtures/empty.md", "--format", "json"]);
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(
        result["diagnostics"]
            .as_array()
            .expect("diagnostics is an array")
            .is_empty()
    )
    .is_equal_to(false);
}

#[test]
fn accepts_a_unicode_document_without_optional_sections() {
    let output = common::output(&["check", "tests/fixtures/minimal.md", "--format", "json"]);
    let (status, report) = common::parse(&output);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["schema_version"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
    assert_that!(output.stderr.is_empty()).is_equal_to(true);
}

#[test]
fn recursively_checks_markdown_and_respects_ignore_files() {
    let workspace = Workspace::new();
    workspace.write("a.md", include_str!("fixtures/minimal.md"));
    workspace.write("nested/b.MD", "");
    workspace.write("ignored.md", "");
    workspace.write(".gitignore", "ignored.md\n");
    let (status, report) = workspace.check(&["."]);
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
    assert_that!(std::fs::read_to_string(workspace.path().join("a.md")).expect("fixture readable"))
        .is_equal_to(include_str!("fixtures/minimal.md"));
}

#[test]
fn reports_invalid_utf8_without_abandoning_other_files() {
    let workspace = Workspace::new();
    workspace.write("broken.md", [0xff, 0xfe]);
    workspace.write("empty.md", "");
    let (status, report) = workspace.check(&["."]);
    let rules = common::rules(&report);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(rules.contains(&"MDS901")).is_equal_to(true);
    assert_that!(rules.contains(&"MDS001")).is_equal_to(true);
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn reports_missing_input_in_json() {
    let workspace = Workspace::new();
    let (status, report) = workspace.check(&["missing.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS901"));
}

#[test]
fn rejects_unknown_configuration_keys_in_json() {
    let workspace = Workspace::new();
    let config = workspace.config("[document]\nrequire_logci = false\n");
    let (status, report) = common::report(&[
        "check",
        "tests/fixtures/minimal.md",
        "--config",
        &config,
        "--format",
        "json",
    ]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS900"));
}

#[test]
fn applies_partial_configuration_defaults() {
    let workspace = Workspace::new();
    let config =
        workspace.config("[document]\nrequire_components = true\nrequire_mappings = true\n");
    let (status, report) = common::report(&[
        "check",
        "tests/fixtures/minimal.md",
        "--config",
        &config,
        "--format",
        "json",
    ]);
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn prints_human_diagnostics_to_stderr() {
    let output = common::output(&["check", "tests/fixtures/empty.md"]);
    assert_that!(output.status.code()).is_equal_to(Some(1));
    assert_that!(output.stdout.is_empty()).is_equal_to(true);
    assert_that!(String::from_utf8_lossy(&output.stderr).contains("ERROR MDS001"))
        .is_equal_to(true);
}

#[test]
fn exposes_help_version_and_usage_exit_codes() {
    let help = common::output(&["--help"]);
    let version = common::output(&["--version"]);
    let invalid = common::output(&["check"]);
    assert_that!(help.status.code()).is_equal_to(Some(0));
    assert_that!(String::from_utf8_lossy(&help.stdout).contains("check")).is_equal_to(true);
    assert_that!(version.status.code()).is_equal_to(Some(0));
    assert_that!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")))
        .is_equal_to(true);
    assert_that!(invalid.status.code()).is_equal_to(Some(2));
}

#[test]
fn validates_inline_targets_when_reference_style_is_disabled() {
    let workspace = Workspace::new();
    let config = workspace.config("[references]\nrequire_reference_style = false\n");
    workspace.write(
        "endpoint.md",
        format!(
            "{}\n[missing](missing.md)\n\n### Ссылки\n",
            include_str!("fixtures/minimal.md")
        ),
    );
    let (status, report) = workspace.check(&["endpoint.md", "--config", &config]);
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS012"));
}
