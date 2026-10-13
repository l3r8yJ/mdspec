#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::{Workspace, rule, rule_count};

const SINGLE: &str = "[document]\nmultiple_endpoints = false\n";
const ENDPOINT: &str = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст [a][a].\n\n";
const SECOND: &str = "## Эндпоинт: Б\n\nPath: `GET /b`\n\n### Логика работы\n\nТекст.\n\n";

#[test]
fn accepts_a_titled_document_with_a_preamble_three_endpoints_and_shared_references() {
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(include_str!("fixtures/endpoints/multiple.md"), "", true))
        .is_empty();
}

#[test]
fn reports_misordered_sections_only_on_the_endpoint_that_has_them() {
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(include_str!("fixtures/endpoints/order.md"), "", true))
        .is_equal_to(vec![rule("MDS004", 39)]);
}

#[test]
fn rejects_a_duplicate_component_only_within_one_endpoint() {
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(include_str!("fixtures/endpoints/duplicate.md"), "", true))
        .is_equal_to(vec![rule("MDS009", 43)]);
}

#[test]
fn reports_every_extra_endpoint_when_multiple_endpoints_are_disabled() {
    let workspace = Workspace::new();
    let diagnostics =
        workspace.check_rules(include_str!("fixtures/endpoints/multiple.md"), SINGLE, true);
    assert_that!(diagnostics).contains_all_of([
        rule("MDS001", 11),
        rule("MDS001", 27),
        rule("MDS001", 51),
    ]);
}

#[test]
fn accepts_references_sections_inside_each_endpoint() {
    let document = format!("{ENDPOINT}### Ссылки\n\n[a]: #эндпоинт-а\n\n{SECOND}");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn resolves_a_link_to_a_repeated_section_of_another_endpoint() {
    let document = format!("{ENDPOINT}{SECOND}## Ссылки\n\n[a]: #логика-работы-1\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn rejects_a_link_to_a_repeated_section_beyond_the_last_endpoint() {
    let document = format!("{ENDPOINT}{SECOND}## Ссылки\n\n[a]: #логика-работы-2\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS013", 19)]);
}

#[test]
fn rejects_shared_references_that_are_not_at_the_end_of_the_document() {
    let document = format!("{ENDPOINT}## Ссылки\n\n[a]: #эндпоинт-а\n\n{SECOND}");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true))
        .contains_all_of([rule("MDS015", 11), rule("MDS015", 13)]);
}

#[test]
fn rejects_links_without_any_references_section() {
    let document = format!("{ENDPOINT}{SECOND}");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).contains(rule("MDS015", 7));
}

#[test]
fn rejects_an_endpoint_heading_without_a_name() {
    let document = "## Эндпоинт:\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст.\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(document, "", true)).is_equal_to(vec![rule("MDS001", 1)]);
}

#[test]
fn requires_at_least_one_endpoint() {
    let document = "# Сервис\n\n## Общие правила\n\nТекст.\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(document, "", true)).is_equal_to(vec![rule("MDS001", 0)]);
}

#[test]
fn accepts_a_document_without_endpoints_when_they_are_optional() {
    let document = "# Сервис\n\n## Общие правила\n\nТекст.\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(document, "[document]\nrequire_endpoint = false\n", true))
        .is_empty();
}

#[test]
fn reports_missing_path_and_logic_on_their_own_endpoint() {
    let document =
        format!("{SECOND}## Эндпоинт: В\n\n### Компоненты\n\n#### К\n\n##### Описание\n\nТ.\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true))
        .is_equal_to(vec![rule("MDS002", 9), rule("MDS003", 9)]);
}

#[test]
fn rejects_a_title_after_the_first_h2() {
    let document = format!("{SECOND}# Поздний заголовок\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS005", 9)]);
}

#[test]
fn rejects_a_section_before_any_endpoint() {
    let document = format!("### Логика работы\n\nТекст.\n\n{SECOND}");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS005", 1)]);
}

#[test]
fn does_not_check_sections_under_a_preamble_heading() {
    let document =
        format!("## Общие правила\n\n### Заметки\n\n#### Подробности\n\n###### Детали\n\n{SECOND}");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn keeps_single_endpoint_references_open_to_the_end_of_the_document() {
    let document = format!("{ENDPOINT}### Ссылки\n\n[a]: #эндпоинт-а\n\n## Конец\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, SINGLE, true))
        .contains_all_of([rule("MDS015", 11), rule("MDS015", 13)]);
}

#[test]
fn ignores_a_document_level_references_heading_for_a_single_endpoint() {
    let document = format!("{ENDPOINT}## Ссылки\n\n[a]: #эндпоинт-а\n");
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(&document, SINGLE, true)).contains(rule("MDS015", 7));
}

#[test]
fn rejects_a_second_endpoint_even_when_both_names_are_valid() {
    let document = format!(
        "{}\n## Эндпоинт: Второй\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) =
        workspace.check_document(&document, "[document]\nmultiple_endpoints = false\n", false);
    assert_that!(rule_count(&report, "MDS001")).is_equal_to(1);
}

#[test]
fn reports_missing_endpoint_path_and_logic_together_for_a_single_endpoint() {
    let workspace = Workspace::new();
    let (_, report) =
        workspace.check_document("", "[document]\nmultiple_endpoints = false\n", false);
    assert_that!(["MDS001", "MDS002", "MDS003"].map(|rule| rule_count(&report, rule)))
        .is_equal_to([1, 1, 1]);
}
