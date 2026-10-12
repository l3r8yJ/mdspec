#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::{Workspace, rule};

const ENDPOINT: &str =
    "## Эндпоинт: Создание\n\nPath: `POST /v1/backups`\n\n### Логика работы\n\nСоздать бэкап.\n\n";
const REQUEST: &str = "#### Запрос\n\n```json\n{\"name\": \"backup\"}\n```\n\n";
const RESPONSE: &str = "#### Ответ\n\n```json\n{\"id\": 1}\n```\n\n";

fn examples(body: &str) -> String {
    format!("{ENDPOINT}### Примеры\n\n{body}")
}

#[test]
fn accepts_a_request_and_a_response_example() {
    let workspace = Workspace::new();
    let document = examples(&format!("{REQUEST}{RESPONSE}"));
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn rejects_an_example_that_is_neither_a_request_nor_a_response() {
    let document = examples("#### Заголовки\n\n```http\nAccept: */*\n```\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS017", 11)]);
}

#[test]
fn rejects_a_second_request_in_one_endpoint() {
    let workspace = Workspace::new();
    let document = examples(&format!("{REQUEST}{REQUEST}"));
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS017", 17)]);
}

#[test]
fn rejects_an_example_without_a_code_block() {
    let document = examples("#### Запрос\n\nТело запроса.\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS017", 11)]);
}

#[test]
fn rejects_an_example_with_an_empty_code_block() {
    let document = examples("#### Запрос\n\n```json\n```\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS017", 11)]);
}

#[test]
fn rejects_an_example_with_an_indented_code_block() {
    let document = examples("#### Запрос\n\n    {\"name\": \"backup\"}\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS017", 11)]);
}

#[test]
fn rejects_an_examples_section_without_examples() {
    let document = format!("{}### Ссылки\n", examples("Пусто.\n\n"));
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS007", 9)]);
}

#[test]
fn rejects_examples_after_references() {
    let document = format!("{ENDPOINT}### Ссылки\n\n### Примеры\n\n{REQUEST}");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).contains(rule("MDS004", 11));
}

#[test]
fn requires_examples_only_when_configured() {
    let config = "[document]\nrequire_examples = true\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(ENDPOINT, config, true))
        .is_equal_to(vec![rule("MDS003", 1)]);
}

#[test]
fn accepts_documents_without_examples_by_default() {
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(ENDPOINT, "", true)).is_empty();
}

#[test]
fn allows_the_same_example_in_different_endpoints() {
    let second = ENDPOINT
        .replace("Создание", "Копия")
        .replace("/v1/backups", "/v1/copies");
    let document = format!("{}{second}### Примеры\n\n{REQUEST}", examples(REQUEST));
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn reads_example_names_from_the_configured_labels() {
    let config =
        "[labels]\nexamples = \"Examples\"\nrequest = \"Request\"\nresponse = \"Response\"\n";
    let document = format!(
        "{ENDPOINT}### Examples\n\n{}",
        REQUEST.replace("Запрос", "Request")
    );
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, config, true)).is_empty();
}

#[test]
fn rejects_a_request_label_equal_to_the_response_label() {
    let config = "[labels]\nrequest = \"Пример\"\nresponse = \"Пример\"\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(ENDPOINT, config, true))
        .is_equal_to(vec![rule("MDS900", 0)]);
}
