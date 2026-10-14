#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::{Workspace, rule};

const PLAIN: &str = "## Эндпоинт: Б\n\nPath: `GET /b`\n\n### Логика работы\n\nТекст.\n\n";
const LINKED: &str =
    "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\n[Проверить права][check].\n\n";
const GENERAL: &str =
    "## Компоненты\n\n### Проверка прав\n\n#### Описание\n\nПроверить доступ.\n\n";
const REFERENCES: &str = "## Ссылки\n\n[check]: #проверка-прав\n";

#[test]
fn accepts_a_general_component_referenced_from_an_endpoint() {
    let workspace = Workspace::new();
    let document = format!("{LINKED}{GENERAL}{REFERENCES}");
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn rejects_a_general_components_section_placed_before_an_endpoint() {
    let workspace = Workspace::new();
    let document = format!("{GENERAL}{LINKED}{REFERENCES}");
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS004", 1)]);
}

#[test]
fn rejects_a_second_general_components_section() {
    let workspace = Workspace::new();
    let document = format!("{LINKED}{GENERAL}{GENERAL}{REFERENCES}");
    assert_that!(workspace.check_rule_names(&document, "", true)).contains("MDS004".to_owned());
}

#[test]
fn rejects_an_empty_general_components_section() {
    let workspace = Workspace::new();
    let document = format!("{PLAIN}## Компоненты\n\nТекст.\n");
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS007", 9)]);
}

#[test]
fn rejects_a_general_component_without_a_description() {
    let workspace = Workspace::new();
    let document =
        format!("{LINKED}## Компоненты\n\n### Проверка прав\n\nПроверить доступ.\n\n{REFERENCES}");
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS008", 11)]);
}

#[test]
fn rejects_duplicate_general_component_names() {
    let workspace = Workspace::new();
    let document =
        format!("{LINKED}{GENERAL}### Проверка прав\n\n#### Описание\n\nЕщё раз.\n\n{REFERENCES}");
    assert_that!(workspace.check_rule_names(&document, "", true)).contains("MDS009".to_owned());
}

#[test]
fn rejects_a_description_nested_one_level_too_deep_in_a_general_component() {
    let workspace = Workspace::new();
    let document = format!(
        "{LINKED}## Компоненты\n\n### Проверка прав\n\n##### Описание\n\nПроверить доступ.\n\n{REFERENCES}"
    );
    assert_that!(workspace.check_rule_names(&document, "", true))
        .contains_all_of(["MDS005".to_owned(), "MDS008".to_owned()]);
}

#[test]
fn accepts_a_document_that_holds_only_general_components() {
    let workspace = Workspace::new();
    let (_, report) =
        workspace.check_document(&format!("# Общие компоненты\n\n{GENERAL}"), "", true);
    assert_that!(common::rule_names(&report)).does_not_contain("MDS001".to_owned());
}

#[test]
fn keeps_reporting_an_extra_h2_in_single_endpoint_mode() {
    let workspace = Workspace::new();
    let document = format!("{PLAIN}{GENERAL}");
    let config = "[document]\nmultiple_endpoints = false\n";
    assert_that!(workspace.check_rule_names(&document, config, true)).contains("MDS001".to_owned());
}

const LOADING: &str =
    "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\n[Загрузить][load].\n\n";
const CHAINED: &str = "## Компоненты\n\n### Проверка прав\n\n#### Описание\n\nПроверить доступ.\n\n### Загрузка\n\n#### Описание\n\nЗагрузить после [проверки][check].\n\n";

#[test]
fn reports_an_unreferenced_general_component() {
    let workspace = Workspace::new();
    let document = format!("{PLAIN}{GENERAL}");
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS018", 11)]);
}

#[test]
fn does_not_count_a_general_component_linking_to_itself() {
    let workspace = Workspace::new();
    let document = format!(
        "{PLAIN}## Компоненты\n\n### Проверка прав\n\n#### Описание\n\nСм. [себя][check].\n\n{REFERENCES}"
    );
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS018", 11)]);
}

#[test]
fn accepts_a_general_component_referenced_only_by_another_general_component() {
    let workspace = Workspace::new();
    let document =
        format!("{LOADING}{CHAINED}## Ссылки\n\n[check]: #проверка-прав\n[load]: #загрузка\n");
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn accepts_a_percent_encoded_reference_to_a_general_component() {
    let workspace = Workspace::new();
    let document = format!(
        "{LINKED}{GENERAL}## Ссылки\n\n[check]: #%D0%BF%D1%80%D0%BE%D0%B2%D0%B5%D1%80%D0%BA%D0%B0-%D0%BF%D1%80%D0%B0%D0%B2\n"
    );
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn accepts_a_shared_components_file_referenced_from_another_file() {
    let workspace = Workspace::new();
    workspace
        .write("components.md", format!("# Общие компоненты\n\n{GENERAL}"))
        .write(
            "endpoint.md",
            format!("{LINKED}## Ссылки\n\n[check]: components.md#проверка-прав\n"),
        );
    let (code, report) = workspace.check(&["."]);
    assert_that!((code, common::rule_names(&report))).is_equal_to((Some(0), vec![]));
}

#[test]
fn reports_a_shared_component_whose_only_reference_is_excluded_from_the_run() {
    let workspace = Workspace::new();
    workspace
        .write("components.md", format!("# Общие компоненты\n\n{GENERAL}"))
        .write(
            "endpoint.md",
            format!("{LINKED}## Ссылки\n\n[check]: components.md#проверка-прав\n"),
        );
    let (_, report) = workspace.check(&["components.md"]);
    assert_that!(common::rules_with_lines(&report)).is_equal_to(vec![rule("MDS018", 5)]);
}
