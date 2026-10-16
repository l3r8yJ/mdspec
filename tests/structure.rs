#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::{Workspace, diagnostics, rule, rule_count, rules_with_lines};

#[test]
fn accepts_complete_document_with_unicode_and_code_blocks() {
    assert_that!(common::exit_and_rule_names(
        "tests/fixtures/structure/complete.md"
    ))
    .is_equal_to((0, vec![]));
}

#[test]
fn accepts_document_without_optional_sections() {
    assert_that!(common::exit_and_rule_names(
        "tests/fixtures/structure/minimal.md"
    ))
    .is_equal_to((0, vec![]));
}

#[test]
fn reports_only_the_missing_endpoint_in_an_empty_document() {
    assert_that!(common::exit_and_rule_names(
        "tests/fixtures/structure/empty.md"
    ))
    .is_equal_to((1, vec!["MDS001".to_owned()]));
}

#[test]
fn rejects_invalid_method_and_section_order() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/order.md");
    assert_that!(rules).contains_all_of(["MDS002".to_owned(), "MDS004".to_owned()]);
}

#[test]
fn reports_component_and_mapping_errors() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/invalid.md");
    assert_that!(rules).contains_all_of([
        "MDS008".to_owned(),
        "MDS009".to_owned(),
        "MDS010".to_owned(),
        "MDS005".to_owned(),
    ]);
}

#[test]
fn does_not_use_a_code_block_as_endpoint_path() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/code-path.md");
    assert_that!(rules).contains("MDS002".to_owned());
}

#[test]
fn reports_a_missing_path_in_nonempty_document() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/missing-path.md");
    assert_that!(rules).contains("MDS002".to_owned());
}

#[test]
fn rejects_empty_required_mapping_cells() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/empty-cell.md");
    assert_that!(rules).contains("MDS010".to_owned());
}

#[test]
fn does_not_use_comments_as_component_description() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/comment-description.md");
    assert_that!(rules).contains("MDS008".to_owned());
}

#[test]
fn rejects_a_table_outside_a_named_mapping() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/orphan-table.md");
    assert_that!(rules).contains("MDS010".to_owned());
}

#[test]
fn permits_shared_sections_when_endpoint_is_optional() {
    let output = common::output(&[
        "check",
        "tests/fixtures/structure/shared.md",
        "--config",
        "tests/fixtures/structure/shared.toml",
        "--format",
        "json",
    ]);
    assert_that!(output.status.code()).is_equal_to(Some(0));
}

#[test]
fn permits_component_prose_starting_with_path() {
    assert_that!(common::exit_and_rule_names(
        "tests/fixtures/structure/component-path-prose.md"
    ))
    .is_equal_to((0, vec![]));
}

#[test]
fn does_not_accept_logic_prose_as_missing_endpoint_path() {
    let (_, rules) = common::exit_and_rule_names("tests/fixtures/structure/late-path.md");
    assert_that!(rules).contains("MDS002".to_owned());
}

#[test]
fn accepts_unknown_sections_only_when_configured() {
    let document = format!(
        "{}\n### Примечания\nДополнение.\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(
        &document,
        "[document]\nallow_unknown_sections = true\n",
        false,
    );
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn rejects_logic_without_body_content() {
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(
        "## Эндпоинт: Пустой\n\nPath: `GET /`\n\n### Логика работы\n",
        "",
        false,
    );
    assert_that!(rule_count(&report, "MDS007")).is_equal_to(1);
}

#[test]
fn rejects_components_and_mappings_without_named_items() {
    let document = format!(
        "{}\n### Компоненты\n\n### Маппинги\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&document, "", false);
    assert_that!(rule_count(&report, "MDS007")).is_equal_to(2);
}

#[test]
fn rejects_unnamed_components_and_descriptions_under_mappings() {
    let document = format!(
        "{}\n### Компоненты\n\n####\n\n##### Описание\n\nТекст.\n\n### Маппинги\n\n#### Поля\n\n##### Описание\n\nТекст.\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&document, "", false);
    assert_that!(rule_count(&report, "MDS005")).is_equal_to(2);
}

#[test]
fn rejects_multiple_nonempty_descriptions_in_a_component() {
    let document = format!(
        "{}\n### Компоненты\n\n#### Проверка\n\n##### Описание\n\nПервое.\n\n##### Описание\n\nВторое.\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&document, "", false);
    assert_that!(rule_count(&report, "MDS008")).is_equal_to(1);
}

#[test]
fn rejects_a_mapping_table_with_no_data_rows() {
    let document = format!(
        "{}\n### Маппинги\n\n#### Поля\n\n| Source | Target |\n| --- | --- |\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&document, "", false);
    assert_that!(rule_count(&report, "MDS010")).is_equal_to(1);
}

#[test]
fn rejects_duplicate_sections_when_order_is_disabled() {
    let document = format!(
        "{}\n### Логика работы\n\nПовтор.\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(
        &document,
        "[document]\nenforce_section_order = false\n",
        false,
    );
    assert_that!(rule_count(&report, "MDS004")).is_equal_to(1);
}

#[test]
fn allows_unique_sections_out_of_order_when_configured() {
    let document = "## Эндпоинт: Список\n\nPath: `GET /`\n\n### Компоненты\n\n#### Проверка\n\n##### Описание\n\nПроверить запрос.\n\n### Логика работы\n\nВернуть список.\n";
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(
        document,
        "[document]\nenforce_section_order = false\n",
        false,
    );
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn retains_section_order_after_an_out_of_order_section() {
    let document = format!(
        "{}\n### Ссылки\n\n### Компоненты\n\n### Маппинги\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&document, "", false);
    assert_that!(rule_count(&report, "MDS004")).is_equal_to(2);
}

#[test]
fn rejects_duplicate_sections_with_default_ordering() {
    let document = format!(
        "{}\n### Логика работы\n\nПовтор.\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&document, "", false);
    assert_that!(rule_count(&report, "MDS004")).is_equal_to(1);
}

#[test]
fn rejects_unknown_endpoint_sections_when_configured() {
    let document = format!(
        "{}\n### Примечания\nДополнение.\n\n## Общие правила\n\n### Заметки\n\nТекст.\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(
        &document,
        "[document]\nallow_unknown_sections = false\n",
        true
    ))
    .is_equal_to(vec![rule("MDS006", 9)]);
}

#[test]
fn rejects_a_structural_heading_inside_a_blockquote() {
    let document = format!(
        "{}\n> ### Примечания\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    let (_, report) = workspace.check_document(&document, "", true);
    assert_that!(rules_with_lines(&report)).is_equal_to(vec![rule("MDS005", 9)]);
    assert_that!(
        diagnostics(&report)
            .iter()
            .filter_map(|diagnostic| diagnostic["message"].as_str())
            .collect::<Vec<_>>()
    )
    .is_equal_to(vec![
        "Structural headings must not occur inside lists or blockquotes",
    ]);
}

#[test]
fn reports_an_unknown_section_after_a_late_title() {
    let document = format!(
        "{}\n# Приложение\n\n### Примечания\n\nТекст.\n",
        include_str!("fixtures/minimal.md")
    );
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(
        &document,
        "[document]\nallow_unknown_sections = false\n",
        true
    ))
    .is_equal_to(vec![rule("MDS005", 9), rule("MDS006", 11)]);
}

#[test]
fn accepts_a_mapping_table_after_a_late_title_as_directly_under_its_heading() {
    let document = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст.\n\n### Маппинги\n\n#### Поля\n\n# Поздний\n\n| Source | Target |\n| --- | --- |\n| a | b |\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(document, "", true)).is_equal_to(vec![rule("MDS005", 13)]);
}

#[test]
fn counts_logic_text_after_a_late_title() {
    let document = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\n# Поздний\n\nТекст.\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(document, "", true)).is_equal_to(vec![rule("MDS005", 7)]);
}

#[test]
fn counts_an_example_code_block_after_a_late_title() {
    let document = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст.\n\n### Примеры\n\n#### Запрос\n\n# Поздний\n\n```json\n{}\n```\n";
    let workspace = Workspace::new();
    assert_that!(workspace.check_rules(document, "", true)).is_equal_to(vec![rule("MDS005", 13)]);
}
