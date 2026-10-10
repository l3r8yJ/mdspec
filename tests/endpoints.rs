#![cfg(test)]

use asserting::prelude::*;
use std::process::Command;

const SINGLE: &str = "[document]\nmultiple_endpoints = false\n";
const ENDPOINT: &str = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст [a][a].\n\n";
const SECOND: &str = "## Эндпоинт: Б\n\nPath: `GET /b`\n\n### Логика работы\n\nТекст.\n\n";

fn check(document: &str, config: &str) -> Vec<(String, u64)> {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let document_path = directory.path().join("service.md");
    let config_path = directory.path().join("mdspec.toml");
    std::fs::write(&document_path, document).expect("document written");
    std::fs::write(&config_path, config).expect("configuration written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(document_path)
        .args(["--strict", "--format", "json", "--config"])
        .arg(config_path)
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    report["diagnostics"]
        .as_array()
        .expect("diagnostics is an array")
        .iter()
        .map(|diagnostic| {
            (
                diagnostic["rule"].as_str().expect("rule string").to_owned(),
                diagnostic["line"].as_u64().unwrap_or_default(),
            )
        })
        .collect()
}

fn rule(name: &str, line: u64) -> (String, u64) {
    (name.to_owned(), line)
}

#[test]
fn accepts_a_titled_document_with_a_preamble_three_endpoints_and_shared_references() {
    assert_that!(check(include_str!("fixtures/endpoints/multiple.md"), "")).is_empty();
}

#[test]
fn reports_misordered_sections_only_on_the_endpoint_that_has_them() {
    assert_that!(check(include_str!("fixtures/endpoints/order.md"), ""))
        .is_equal_to(vec![rule("MDS004", 39)]);
}

#[test]
fn rejects_a_duplicate_component_only_within_one_endpoint() {
    assert_that!(check(include_str!("fixtures/endpoints/duplicate.md"), ""))
        .is_equal_to(vec![rule("MDS009", 43)]);
}

#[test]
fn reports_every_extra_endpoint_when_multiple_endpoints_are_disabled() {
    let diagnostics = check(include_str!("fixtures/endpoints/multiple.md"), SINGLE);
    assert_that!(diagnostics).contains_all_of([
        rule("MDS001", 11),
        rule("MDS001", 27),
        rule("MDS001", 51),
    ]);
}

#[test]
fn accepts_references_sections_inside_each_endpoint() {
    let document = format!("{ENDPOINT}### Ссылки\n\n[a]: #эндпоинт-а\n\n{SECOND}");
    assert_that!(check(&document, "")).is_empty();
}

#[test]
fn rejects_shared_references_that_are_not_at_the_end_of_the_document() {
    let document = format!("{ENDPOINT}## Ссылки\n\n[a]: #эндпоинт-а\n\n{SECOND}");
    assert_that!(check(&document, "")).contains_all_of([rule("MDS015", 11), rule("MDS015", 13)]);
}

#[test]
fn rejects_links_without_any_references_section() {
    let document = format!("{ENDPOINT}{SECOND}");
    assert_that!(check(&document, "")).contains(rule("MDS015", 7));
}

#[test]
fn rejects_an_endpoint_heading_without_a_name() {
    let document = "## Эндпоинт:\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст.\n";
    assert_that!(check(document, "")).is_equal_to(vec![rule("MDS001", 1)]);
}

#[test]
fn requires_at_least_one_endpoint() {
    let document = "# Сервис\n\n## Общие правила\n\nТекст.\n";
    assert_that!(check(document, "")).is_equal_to(vec![rule("MDS001", 0)]);
}

#[test]
fn accepts_a_document_without_endpoints_when_they_are_optional() {
    let document = "# Сервис\n\n## Общие правила\n\nТекст.\n";
    assert_that!(check(document, "[document]\nrequire_endpoint = false\n")).is_empty();
}

#[test]
fn reports_missing_path_and_logic_on_their_own_endpoint() {
    let document =
        format!("{SECOND}## Эндпоинт: В\n\n### Компоненты\n\n#### К\n\n##### Описание\n\nТ.\n");
    assert_that!(check(&document, "")).is_equal_to(vec![rule("MDS002", 9), rule("MDS003", 9)]);
}

#[test]
fn rejects_a_title_after_the_first_h2() {
    let document = format!("{SECOND}# Поздний заголовок\n");
    assert_that!(check(&document, "")).is_equal_to(vec![rule("MDS005", 9)]);
}

#[test]
fn rejects_a_section_before_any_endpoint() {
    let document = format!("### Логика работы\n\nТекст.\n\n{SECOND}");
    assert_that!(check(&document, "")).is_equal_to(vec![rule("MDS005", 1)]);
}

#[test]
fn does_not_check_sections_under_a_preamble_heading() {
    let document =
        format!("## Общие правила\n\n### Заметки\n\n#### Подробности\n\n###### Детали\n\n{SECOND}");
    assert_that!(check(&document, "")).is_empty();
}

#[test]
fn keeps_single_endpoint_references_open_to_the_end_of_the_document() {
    let document = format!("{ENDPOINT}### Ссылки\n\n[a]: #эндпоинт-а\n\n## Конец\n");
    assert_that!(check(&document, SINGLE))
        .contains_all_of([rule("MDS015", 11), rule("MDS015", 13)]);
}

#[test]
fn ignores_a_document_level_references_heading_for_a_single_endpoint() {
    let document = format!("{ENDPOINT}## Ссылки\n\n[a]: #эндпоинт-а\n");
    assert_that!(check(&document, SINGLE)).contains(rule("MDS015", 7));
}
