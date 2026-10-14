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
