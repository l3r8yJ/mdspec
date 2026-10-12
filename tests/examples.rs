#![cfg(test)]

use asserting::prelude::*;
use std::process::Command;

const ENDPOINT: &str =
    "## Эндпоинт: Создание\n\nPath: `POST /v1/backups`\n\n### Логика работы\n\nСоздать бэкап.\n\n";
const REQUEST: &str = "#### Запрос\n\n```json\n{\"name\": \"backup\"}\n```\n\n";
const RESPONSE: &str = "#### Ответ\n\n```json\n{\"id\": 1}\n```\n\n";

fn check(document: &str, config: &str) -> Vec<(String, u64)> {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let document_path = directory.path().join("endpoint.md");
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

fn examples(body: &str) -> String {
    format!("{ENDPOINT}### Примеры\n\n{body}")
}

#[test]
fn accepts_a_request_and_a_response_example() {
    assert_that!(check(&examples(&format!("{REQUEST}{RESPONSE}")), "")).is_empty();
}

#[test]
fn rejects_an_example_that_is_neither_a_request_nor_a_response() {
    let document = examples("#### Заголовки\n\n```http\nAccept: */*\n```\n");
    assert_that!(check(&document, "")).is_equal_to(vec![rule("MDS017", 11)]);
}

#[test]
fn rejects_a_second_request_in_one_endpoint() {
    assert_that!(check(&examples(&format!("{REQUEST}{REQUEST}")), ""))
        .is_equal_to(vec![rule("MDS017", 17)]);
}

#[test]
fn rejects_an_example_without_a_code_block() {
    let document = examples("#### Запрос\n\nТело запроса.\n");
    assert_that!(check(&document, "")).is_equal_to(vec![rule("MDS017", 11)]);
}

#[test]
fn rejects_an_example_with_an_empty_code_block() {
    let document = examples("#### Запрос\n\n```json\n```\n");
    assert_that!(check(&document, "")).is_equal_to(vec![rule("MDS017", 11)]);
}

#[test]
fn rejects_an_example_with_an_indented_code_block() {
    let document = examples("#### Запрос\n\n    {\"name\": \"backup\"}\n");
    assert_that!(check(&document, "")).is_equal_to(vec![rule("MDS017", 11)]);
}

#[test]
fn rejects_an_examples_section_without_examples() {
    let document = format!("{}### Ссылки\n", examples("Пусто.\n\n"));
    assert_that!(check(&document, "")).is_equal_to(vec![rule("MDS007", 9)]);
}

#[test]
fn rejects_examples_after_references() {
    let document = format!("{ENDPOINT}### Ссылки\n\n### Примеры\n\n{REQUEST}");
    assert_that!(check(&document, "")).contains(rule("MDS004", 11));
}

#[test]
fn requires_examples_only_when_configured() {
    let config = "[document]\nrequire_examples = true\n";
    assert_that!(check(ENDPOINT, config)).is_equal_to(vec![rule("MDS003", 1)]);
}

#[test]
fn accepts_documents_without_examples_by_default() {
    assert_that!(check(ENDPOINT, "")).is_empty();
}

#[test]
fn allows_the_same_example_in_different_endpoints() {
    let second = ENDPOINT
        .replace("Создание", "Копия")
        .replace("/v1/backups", "/v1/copies");
    let document = format!("{}{second}### Примеры\n\n{REQUEST}", examples(REQUEST));
    assert_that!(check(&document, "")).is_empty();
}

#[test]
fn reads_example_names_from_the_configured_labels() {
    let config =
        "[labels]\nexamples = \"Examples\"\nrequest = \"Request\"\nresponse = \"Response\"\n";
    let document = format!(
        "{ENDPOINT}### Examples\n\n{}",
        REQUEST.replace("Запрос", "Request")
    );
    assert_that!(check(&document, config)).is_empty();
}

#[test]
fn rejects_a_request_label_equal_to_the_response_label() {
    let config = "[labels]\nrequest = \"Пример\"\nresponse = \"Пример\"\n";
    assert_that!(check(ENDPOINT, config)).is_equal_to(vec![rule("MDS900", 0)]);
}
