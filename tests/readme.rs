#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::Workspace;

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

#[test]
fn accepts_the_quick_start_document() {
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(
        block_after("Write a document:", "```md"),
        block_after("save this as `mdspec.toml`:", "```toml"),
        true,
    );
    assert_that!(report["diagnostics"].clone()).is_equal_to(serde_json::json!([]));
}

#[test]
fn accepts_the_quick_start_document_with_the_documented_examples() {
    let document = format!(
        "{}\n\n{}",
        block_after("Write a document:", "```md"),
        block_after("### MDS017 Examples", "````md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(
        &document,
        block_after("save this as `mdspec.toml`:", "```toml"),
        true,
    );
    assert_that!(report["diagnostics"].clone()).is_equal_to(serde_json::json!([]));
}

#[test]
fn accepts_the_documented_general_component_example() {
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(
        block_after("### MDS018 Referenced general components", "```md"),
        block_after("save this as `mdspec.toml`:", "```toml"),
        true,
    );
    assert_that!(report["diagnostics"].clone()).is_equal_to(serde_json::json!([]));
}
