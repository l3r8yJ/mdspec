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
fn rejects_a_general_components_section_that_holds_only_references() {
    let workspace = Workspace::new();
    let document = format!(
        "{LINKED}## Компоненты

### Ссылки

[check]: #эндпоинт-а
"
    );
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS007", 9)]);
}

#[test]
fn rejects_a_general_components_section_that_holds_only_an_empty_references_block() {
    let workspace = Workspace::new();
    let document = format!("{PLAIN}## Компоненты\n\n### Ссылки\n");
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

const OWNER: &str = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст.\n\n### Компоненты\n\n#### Загрузка\n\n##### Описание\n\nЗагрузить данные.\n\n";
const BORROWER: &str =
    "## Эндпоинт: Б\n\nPath: `GET /b`\n\n### Логика работы\n\n[Загрузить][load].\n\n";
const LOAD: &str = "## Ссылки\n\n[load]: #загрузка\n";

#[test]
fn accepts_an_internal_component_referenced_from_its_own_endpoint() {
    let workspace = Workspace::new();
    let owner = OWNER.replace("Текст.", "[Загрузить][load].");
    let document = format!("{owner}{LOAD}");
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn reports_an_internal_component_referenced_from_another_endpoint_and_suggests_the_fix() {
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&format!("{OWNER}{BORROWER}{LOAD}"), "", true);
    assert_that!(common::rules_with_lines(&report)).is_equal_to(vec![rule("MDS019", 23)]);
    assert_that!(
        report["diagnostics"][0]["help"]
            .as_str()
            .unwrap_or_default()
    )
    .contains("Компоненты");
}

#[test]
fn reports_an_internal_component_referenced_from_a_general_component() {
    let workspace = Workspace::new();
    let general =
        "## Компоненты\n\n### Проверка прав\n\n#### Описание\n\nПеред [загрузкой][load].\n\n";
    let owner = OWNER.replace("Текст.", "[Проверить][check].");
    let document =
        format!("{owner}{general}## Ссылки\n\n[load]: #загрузка\n[check]: #проверка-прав\n");
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS019", 23)]);
}

#[test]
fn reports_an_internal_component_referenced_from_another_file() {
    let workspace = Workspace::new();
    workspace.write("a.md", OWNER).write(
        "b.md",
        format!("{BORROWER}## Ссылки\n\n[load]: a.md#загрузка\n"),
    );
    let (_, report) = workspace.check(&["."]);
    let located: Vec<_> = common::diagnostics(&report)
        .iter()
        .map(|diagnostic| {
            (
                diagnostic["rule"].as_str(),
                diagnostic["file"]
                    .as_str()
                    .map(|file| file.ends_with("b.md")),
                diagnostic["line"].as_u64(),
            )
        })
        .collect();
    assert_that!(located).is_equal_to(vec![(Some("MDS019"), Some(true), Some(7))]);
}

#[test]
fn tells_apart_internal_components_that_share_a_name_across_endpoints() {
    let workspace = Workspace::new();
    let second = format!(
        "{BORROWER}### Компоненты\n\n#### Загрузка\n\n##### Описание\n\nСвоя загрузка.\n\n"
    );
    let own = format!("{OWNER}{second}## Ссылки\n\n[load]: #загрузка-1\n");
    let foreign = format!("{OWNER}{second}{LOAD}");
    assert_that!(workspace.check_rules(&own, "", true)).is_empty();
    assert_that!(workspace.check_rules(&foreign, "", true)).is_equal_to(vec![rule("MDS019", 23)]);
}

#[test]
fn prints_the_suggested_fix_in_text_output() {
    let workspace = Workspace::new();
    workspace.write("endpoint.md", format!("{OWNER}{BORROWER}{LOAD}"));
    let output = workspace.output(&["check", "endpoint.md"]);
    assert_that!(String::from_utf8_lossy(&output.stderr).into_owned())
        .contains("help: Move Загрузка to the H2 Компоненты section or remove the link");
}

#[test]
fn ignores_an_h4_heading_outside_the_components_section_when_scoping_internal_components() {
    let workspace = Workspace::new();
    let owner = PLAIN.replace("Текст.", "Текст.\n\n#### Загрузка\n\nТекст.");
    let borrower = LINKED.replace("Проверить права][check]", "Загрузка][load]");
    let document = format!("{owner}{borrower}## Ссылки\n\n[load]: #загрузка\n");
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS005", 9)]);
}

#[test]
fn names_the_expected_heading_level_of_a_missing_component_description() {
    let workspace = Workspace::new();
    let internal = PLAIN.replace(
        "Текст.",
        "Текст.\n\n### Компоненты\n\n#### Загрузка\n\nТекст.",
    );
    let document =
        format!("{internal}{LINKED}## Компоненты\n\n### Проверка прав\n\nТекст.\n\n{REFERENCES}");
    let (_, report) = workspace.check_document(&document, "", true);
    let messages: Vec<_> = common::diagnostics(&report)
        .iter()
        .filter(|diagnostic| diagnostic["rule"] == "MDS008")
        .map(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .unwrap_or_default()
                .to_owned()
        })
        .collect();
    assert_that!(messages).is_equal_to(vec![
        "Component Загрузка is missing H5 Описание".to_owned(),
        "Component Проверка прав is missing H4 Описание".to_owned(),
    ]);
}

#[test]
fn rejects_an_empty_description_followed_by_a_sibling_heading_with_text() {
    let workspace = Workspace::new();
    let general =
        "## Компоненты\n\n### Проверка прав\n\n#### Описание\n\n#### Детали\n\nТекст.\n\n";
    let document = format!("{LINKED}{general}{REFERENCES}");
    assert_that!(workspace.check_rules(&document, "", true))
        .is_equal_to(vec![rule("MDS008", 13), rule("MDS005", 15)]);
}

#[test]
fn treats_a_references_heading_inside_the_general_section_as_the_definitions_block() {
    let workspace = Workspace::new();
    let document = format!("{LINKED}{GENERAL}### Ссылки\n\n[check]: #проверка-прав\n");
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn still_checks_an_internal_component_named_like_the_references_block() {
    let workspace = Workspace::new();
    let owner = OWNER.replace(
        "#### Загрузка\n\n##### Описание\n\nЗагрузить данные.\n\n",
        "#### Ссылки\n\nТекст.\n\n",
    );
    let borrower = BORROWER.replace("Загрузить", "Ссылки");
    let document = format!("{owner}{borrower}## Ссылки\n\n[load]: #ссылки\n");
    assert_that!(workspace.check_rule_names(&document, "", true)).contains("MDS008".to_owned());
}

#[test]
fn reports_an_internal_component_linked_from_a_preamble_or_a_non_endpoint_section() {
    let workspace = Workspace::new();
    let preamble =
        "# Сервис\n\nСм. [загрузку][load].\n\n## Общие правила\n\nИ ещё [раз][load].\n\n";
    let document = format!("{preamble}{OWNER}{LOAD}");
    assert_that!(workspace.check_rules(&document, "", true))
        .is_equal_to(vec![rule("MDS019", 3), rule("MDS019", 7)]);
}

#[test]
fn does_not_declare_internal_components_under_a_non_endpoint_section() {
    let workspace = Workspace::new();
    let shared =
        "## Общие правила\n\n### Компоненты\n\n#### Проверка прав\n\nПроверить доступ.\n\n";
    let document = format!("{LINKED}{shared}{REFERENCES}");
    assert_that!(workspace.check_rules(&document, "", true)).is_empty();
}

#[test]
fn rejects_a_general_component_heading_without_a_name() {
    let workspace = Workspace::new();
    let document =
        format!("{LINKED}## Компоненты\n\n### \n\n#### Описание\n\nТекст.\n\n{REFERENCES}");
    assert_that!(workspace.check_rules(&document, "", true)).contains(rule("MDS005", 11));
}

#[test]
fn does_not_count_an_image_as_a_general_component_reference() {
    let workspace = Workspace::new();
    let plain = PLAIN.replace("Текст.", "![Схема](#проверка-прав)");
    let document = format!("{plain}{GENERAL}");
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![rule("MDS018", 11)]);
}

#[test]
fn keeps_an_internal_component_after_a_late_title_scoped_to_its_endpoint() {
    let workspace = Workspace::new();
    let owner = OWNER.replace("### Компоненты", "# Приложение\n\n### Компоненты");
    let document = format!("{owner}{BORROWER}{LOAD}");
    assert_that!(workspace.check_rules(&document, "", true))
        .is_equal_to(vec![rule("MDS005", 9), rule("MDS019", 25)]);
}

#[test]
fn declares_general_components_after_a_late_title() {
    let workspace = Workspace::new();
    let document = format!(
        "{PLAIN}## Компоненты\n\n### А\n\n#### Описание\n\nТ.\n\n# Приложение\n\n### Б\n\nТ.\n"
    );
    assert_that!(workspace.check_rules(&document, "", true)).is_equal_to(vec![
        rule("MDS018", 11),
        rule("MDS005", 17),
        rule("MDS008", 19),
        rule("MDS018", 19),
    ]);
}

#[test]
fn counts_description_text_after_a_late_title() {
    let workspace = Workspace::new();
    let document =
        format!("{PLAIN}## Компоненты\n\n### А\n\n#### Описание\n\n# Приложение\n\nТ.\n");
    assert_that!(workspace.check_rules(&document, "", true))
        .is_equal_to(vec![rule("MDS018", 11), rule("MDS005", 15)]);
}
