#![cfg(test)]

use asserting::prelude::*;
use std::process::Command;

fn check(document: &str, config: &str) -> serde_json::Value {
    let directory = tempfile::tempdir().expect("temporary directory created");
    let document_path = directory.path().join("endpoint.md");
    let config_path = directory.path().join("mdspec.toml");
    std::fs::write(&document_path, document).expect("document written");
    std::fs::write(&config_path, config).expect("configuration written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(document_path)
        .args(["--format", "json", "--config"])
        .arg(config_path)
        .output()
        .expect("the built binary starts");
    serde_json::from_slice(&output.stdout).expect("stdout is JSON")
}

fn rule_count(report: &serde_json::Value, rule: &str) -> usize {
    report["diagnostics"]
        .as_array()
        .expect("diagnostics is an array")
        .iter()
        .filter(|diagnostic| diagnostic["rule"] == rule)
        .count()
}

#[test]
fn rejects_a_second_endpoint_even_when_both_names_are_valid() {
    let document = format!(
        "{}\n## Эндпоинт: Второй\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "[document]\nmultiple_endpoints = false\n");
    assert_that!(rule_count(&report, "MDS001")).is_equal_to(1);
}

#[test]
fn reports_missing_endpoint_path_and_logic_together_for_a_single_endpoint() {
    let report = check("", "[document]\nmultiple_endpoints = false\n");
    assert_that!(["MDS001", "MDS002", "MDS003"].map(|rule| rule_count(&report, rule)))
        .is_equal_to([1, 1, 1]);
}

#[test]
fn accepts_unknown_sections_only_when_configured() {
    let document = format!(
        "{}\n### Примечания\nДополнение.\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "[document]\nallow_unknown_sections = true\n");
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn rejects_logic_without_body_content() {
    let report = check(
        "## Эндпоинт: Пустой\n\nPath: `GET /`\n\n### Логика работы\n",
        "",
    );
    assert_that!(rule_count(&report, "MDS007")).is_equal_to(1);
}

#[test]
fn rejects_components_and_mappings_without_named_items() {
    let document = format!(
        "{}\n### Компоненты\n\n### Маппинги\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "");
    assert_that!(rule_count(&report, "MDS007")).is_equal_to(2);
}

#[test]
fn rejects_unnamed_components_and_descriptions_under_mappings() {
    let document = format!(
        "{}\n### Компоненты\n\n####\n\n##### Описание\n\nТекст.\n\n### Маппинги\n\n#### Поля\n\n##### Описание\n\nТекст.\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "");
    assert_that!(rule_count(&report, "MDS005")).is_equal_to(2);
}

#[test]
fn rejects_multiple_nonempty_descriptions_in_a_component() {
    let document = format!(
        "{}\n### Компоненты\n\n#### Проверка\n\n##### Описание\n\nПервое.\n\n##### Описание\n\nВторое.\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "");
    assert_that!(rule_count(&report, "MDS008")).is_equal_to(1);
}

#[test]
fn rejects_a_mapping_table_with_no_data_rows() {
    let document = format!(
        "{}\n### Маппинги\n\n#### Поля\n\n| Source | Target |\n| --- | --- |\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "");
    assert_that!(rule_count(&report, "MDS010")).is_equal_to(1);
}

#[test]
fn rejects_duplicate_sections_when_order_is_disabled() {
    let document = format!(
        "{}\n### Логика работы\n\nПовтор.\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "[document]\nenforce_section_order = false\n");
    assert_that!(rule_count(&report, "MDS004")).is_equal_to(1);
}

#[test]
fn allows_unique_sections_out_of_order_when_configured() {
    let document = "## Эндпоинт: Список\n\nPath: `GET /`\n\n### Компоненты\n\n#### Проверка\n\n##### Описание\n\nПроверить запрос.\n\n### Логика работы\n\nВернуть список.\n";
    let report = check(document, "[document]\nenforce_section_order = false\n");
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn retains_section_order_after_an_out_of_order_section() {
    let document = format!(
        "{}\n### Ссылки\n\n### Компоненты\n\n### Маппинги\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "");
    assert_that!(rule_count(&report, "MDS004")).is_equal_to(2);
}

#[test]
fn rejects_duplicate_sections_with_default_ordering() {
    let document = format!(
        "{}\n### Логика работы\n\nПовтор.\n",
        include_str!("fixtures/minimal.md")
    );
    let report = check(&document, "");
    assert_that!(rule_count(&report, "MDS004")).is_equal_to(1);
}
