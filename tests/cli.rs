use asserting::prelude::*;
use std::process::Command;

#[test]
fn rejects_empty_document_with_machine_readable_diagnostics() {
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args(["check", "tests/fixtures/empty.md", "--format", "json"])
        .output()
        .expect("the built binary starts");
    assert_that!(output.status.code()).is_equal_to(Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
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
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args(["check", "tests/fixtures/minimal.md", "--format", "json"])
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_that!(output.status.code()).is_equal_to(Some(0));
    assert_that!(report["schema_version"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
    assert_that!(output.stderr.is_empty()).is_equal_to(true);
}

#[test]
fn recursively_checks_markdown_and_respects_ignore_files() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    std::fs::create_dir(directory.path().join("nested")).expect("nested directory created");
    std::fs::write(
        directory.path().join("a.md"),
        include_str!("fixtures/minimal.md"),
    )
    .expect("fixture written");
    std::fs::write(directory.path().join("nested/b.MD"), "").expect("empty fixture written");
    std::fs::write(directory.path().join("ignored.md"), "").expect("ignored fixture written");
    std::fs::write(directory.path().join(".gitignore"), "ignored.md\n").expect("ignore written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(directory.path())
        .args(["--format", "json"])
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_that!(output.status.code()).is_equal_to(Some(1));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
    assert_that!(std::fs::read_to_string(directory.path().join("a.md")).expect("fixture readable"))
        .is_equal_to(include_str!("fixtures/minimal.md"));
}

#[test]
fn reports_invalid_utf8_without_abandoning_other_files() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    std::fs::write(directory.path().join("broken.md"), [0xff, 0xfe]).expect("invalid UTF8 written");
    std::fs::write(directory.path().join("empty.md"), "").expect("empty fixture written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(directory.path())
        .args(["--format", "json"])
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    let rules: Vec<_> = report["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .filter_map(|item| item["rule"].as_str())
        .collect();
    assert_that!(output.status.code()).is_equal_to(Some(2));
    assert_that!(rules.contains(&"MDS901")).is_equal_to(true);
    assert_that!(rules.contains(&"MDS001")).is_equal_to(true);
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn reports_missing_input_in_json() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(directory.path().join("missing.md"))
        .args(["--format", "json"])
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_that!(output.status.code()).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS901"));
}

#[test]
fn rejects_unknown_configuration_keys_in_json() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let config = directory.path().join("mdspec.toml");
    std::fs::write(&config, "[document]\nrequire_logci = false\n").expect("configuration written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args([
            "check",
            "tests/fixtures/minimal.md",
            "--format",
            "json",
            "--config",
        ])
        .arg(config)
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_that!(output.status.code()).is_equal_to(Some(2));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS900"));
}

#[test]
fn applies_partial_configuration_defaults() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let config = directory.path().join("mdspec.toml");
    std::fs::write(
        &config,
        "[document]\nrequire_components = true\nrequire_mappings = true\n",
    )
    .expect("configuration written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args([
            "check",
            "tests/fixtures/minimal.md",
            "--format",
            "json",
            "--config",
        ])
        .arg(config)
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_that!(output.status.code()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn prints_human_diagnostics_to_stderr() {
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .args(["check", "tests/fixtures/empty.md"])
        .output()
        .expect("the built binary starts");
    assert_that!(output.status.code()).is_equal_to(Some(1));
    assert_that!(output.stdout.is_empty()).is_equal_to(true);
    assert_that!(String::from_utf8_lossy(&output.stderr).contains("ERROR MDS001"))
        .is_equal_to(true);
}

#[test]
fn exposes_help_version_and_usage_exit_codes() {
    let help = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("--help")
        .output()
        .expect("binary starts");
    let version = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("--version")
        .output()
        .expect("binary starts");
    let invalid = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .output()
        .expect("binary starts");
    assert_that!(help.status.code()).is_equal_to(Some(0));
    assert_that!(String::from_utf8_lossy(&help.stdout).contains("check")).is_equal_to(true);
    assert_that!(version.status.code()).is_equal_to(Some(0));
    assert_that!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")))
        .is_equal_to(true);
    assert_that!(invalid.status.code()).is_equal_to(Some(2));
}

#[test]
fn validates_inline_targets_when_reference_style_is_disabled() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let config = directory.path().join("mdspec.toml");
    let document = directory.path().join("endpoint.md");
    std::fs::write(&config, "[references]\nrequire_reference_style = false\n")
        .expect("configuration written");
    std::fs::write(
        &document,
        format!(
            "{}\n[missing](missing.md)\n\n### Ссылки\n",
            include_str!("fixtures/minimal.md")
        ),
    )
    .expect("document written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(document)
        .args(["--format", "json", "--config"])
        .arg(config)
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_that!(output.status.code()).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS012"));
}
