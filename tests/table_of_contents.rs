#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::Workspace;

const ALLOWED: &str = "[references]\ntoc_allowed = true\n";
const ENDPOINT: &str = "## Эндпоинт: А\n\nPath: `GET /a`\n\n### Логика работы\n\nТекст.\n";

#[test]
fn accepts_a_standalone_toc_marker_when_allowed() {
    let workspace = Workspace::new();
    let document = format!("[TOC]\n\n{ENDPOINT}");
    assert_that!(workspace.check_rule_names(&document, ALLOWED, true)).is_empty();
}

#[test]
fn accepts_the_gitlab_toc_marker_when_allowed() {
    let workspace = Workspace::new();
    let document = format!("[[_TOC_]]\n\n{ENDPOINT}");
    assert_that!(workspace.check_rule_names(&document, ALLOWED, true)).is_empty();
}

#[test]
fn reports_a_toc_reference_inside_a_sentence_even_when_allowed() {
    let workspace = Workspace::new();
    let document = format!("Смотри [TOC] ниже.\n\n{ENDPOINT}");
    assert_that!(workspace.check_rule_names(&document, ALLOWED, true))
        .contains_exactly(["MDS011".to_owned(), "MDS015".to_owned()]);
}

#[test]
fn reports_a_toc_marker_inside_a_blockquote_even_when_allowed() {
    let workspace = Workspace::new();
    let document = format!("> [TOC]\n\n{ENDPOINT}");
    assert_that!(workspace.check_rule_names(&document, ALLOWED, true))
        .contains_exactly(["MDS011".to_owned(), "MDS015".to_owned()]);
}

#[test]
fn reports_a_standalone_toc_marker_by_default() {
    let workspace = Workspace::new();
    let document = format!("[TOC]\n\n{ENDPOINT}");
    assert_that!(workspace.check_rule_names(&document, "", true))
        .contains_exactly(["MDS011".to_owned(), "MDS015".to_owned()]);
}

#[test]
fn keeps_a_defined_toc_reference_as_a_link_when_allowed() {
    let workspace = Workspace::new();
    let document = format!("[TOC]\n\n{ENDPOINT}\n### Ссылки\n\n[TOC]: #эндпоинт-а\n");
    assert_that!(workspace.check_rule_names(&document, ALLOWED, true)).is_empty();
}
