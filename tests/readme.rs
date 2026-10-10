#![cfg(test)]

use asserting::prelude::*;
use std::process::Command;

const README: &str = include_str!("../README.md");

fn block_after<'a>(marker: &str, fence: &str) -> &'a str {
    let rest = README
        .split_once(marker)
        .expect("README contains the marker")
        .1;
    let body = rest
        .split_once(&format!("{fence}\n"))
        .expect("a fenced block follows the marker")
        .1;
    body.split_once(&format!(
        "\n{}\n",
        fence.trim_end_matches(char::is_alphanumeric)
    ))
    .expect("the fenced block is closed")
    .0
}

fn check(document: &str) -> serde_json::Value {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let document_path = directory.path().join("endpoint.md");
    let config_path = directory.path().join("mdspec.toml");
    std::fs::write(&document_path, document).expect("document written");
    std::fs::write(
        &config_path,
        block_after("save this as `mdspec.toml`:", "```toml"),
    )
    .expect("configuration written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(document_path)
        .args(["--strict", "--format", "json", "--config"])
        .arg(config_path)
        .output()
        .expect("the built binary starts");
    serde_json::from_slice(&output.stdout).expect("stdout is JSON")
}

#[test]
fn accepts_the_quick_start_document() {
    let report = check(block_after("Write a document:", "```md"));
    assert_that!(report["diagnostics"].clone()).is_equal_to(serde_json::json!([]));
}

#[test]
fn accepts_the_quick_start_document_with_the_documented_examples() {
    let document = format!(
        "{}\n\n{}",
        block_after("Write a document:", "```md"),
        block_after("### MDS017 Examples", "````md")
    );
    assert_that!(check(&document)["diagnostics"].clone()).is_equal_to(serde_json::json!([]));
}
