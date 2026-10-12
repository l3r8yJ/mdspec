#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::Workspace;

#[test]
fn checks_multiple_explicit_files_in_one_invocation() {
    let workspace = Workspace::with_documents(&["first.md", "second.md"]);
    let (status, report) = workspace.check(&["first.md", "second.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn single_star_does_not_cross_directory_boundaries() {
    let workspace = Workspace::with_documents(&[
        "docs/api/service/one.md",
        "docs/team/api/service/two.md",
        "docs/api/service/nested/three.md",
    ]);
    let (status, report) = workspace.check(&["docs/*/service/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn recursive_glob_handles_zero_or_multiple_directory_segments() {
    let workspace = Workspace::with_documents(&[
        "docs/service/direct.md",
        "docs/team/api/service/nested/deep.md",
    ]);
    let (status, report) = workspace.check(&["docs/**/service/**/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn exclusions_win_over_all_inputs_and_skip_matching_directory_contents() {
    let workspace = Workspace::with_documents(&[
        "docs/kept.md",
        "docs/generated/nested/skipped.md",
        "docs/private.md",
    ]);
    let (status, report) = workspace.check(&[
        "docs",
        "docs/private.md",
        "--exclude",
        "docs/generated",
        "--exclude",
        "docs/private.md",
    ]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn exclusion_globs_use_the_same_segment_and_recursive_matching() {
    let workspace = Workspace::with_documents(&[
        "docs/a/service/kept.md",
        "docs/a/service/generated/deep.md",
        "docs/a/service/old.md",
    ]);
    let (status, report) = workspace.check(&[
        "docs/**/*.md",
        "--exclude",
        "docs/**/generated",
        "--exclude",
        "docs/*/service/old.md",
    ]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn directory_and_glob_discovery_respect_ignore_and_hidden_files() {
    let workspace =
        Workspace::with_documents(&["docs/kept.md", "docs/ignored.md", "docs/.hidden.md"]);
    workspace.write(".gitignore", "ignored.md\n");
    let (status, report) = workspace.check(&["docs", "docs/**/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn explicit_files_bypass_ignore_and_hidden_discovery_rules() {
    let workspace = Workspace::with_documents(&["ignored.md", ".hidden.md"]);
    workspace.write(".ignore", "ignored.md\n");
    let (status, report) = workspace.check(&["ignored.md", ".hidden.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn overlapping_inputs_and_normalized_paths_validate_each_file_once() {
    let workspace = Workspace::with_documents(&["docs/first.md", "docs/sub/second.md"]);
    let absolute = workspace.path().join("docs/first.md");
    let (status, report) = workspace.check(&[
        "docs",
        "./docs/**/*.md",
        "docs/sub/../first.md",
        absolute.to_str().expect("temporary path is UTF8"),
    ]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn existing_literal_paths_with_brackets_take_precedence_over_globs() {
    let workspace = Workspace::with_documents(&["docs/[ab].md", "docs/a.md"]);
    let (status, report) = workspace.check(&["docs/[ab].md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn absolute_globs_and_relative_exclusions_share_path_identity() {
    let workspace =
        Workspace::with_documents(&["доки с пробелом/первый.md", "доки с пробелом/второй.md"]);
    let pattern = workspace.path().join("доки с пробелом/*.md");
    let (status, report) = workspace.check(&[
        pattern.to_str().expect("temporary path is UTF8"),
        "--exclude",
        "./доки с пробелом/первый.md",
    ]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn invalid_and_unmatched_inputs_do_not_abandon_valid_documents() {
    let workspace = Workspace::new();
    workspace.write("empty.md", "");
    let (status, report) =
        workspace.check(&["empty.md", "missing.md", "missing/**/*.md", "docs/["]);
    let diagnostics = report["diagnostics"].as_array().expect("diagnostic array");
    let operational = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic["rule"]
                .as_str()
                .is_some_and(|rule| rule.starts_with("MDS9"))
        })
        .count();
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(operational).is_equal_to(3);
    assert_that!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["rule"] == "MDS001")
    )
    .is_equal_to(true);
}

#[test]
fn invalid_exclusions_report_an_error_without_abandoning_valid_inputs() {
    let workspace = Workspace::with_documents(&["valid.md"]);
    let (status, report) = workspace.check(&["valid.md", "--exclude", "docs/["]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn unmatched_exclusions_are_valid() {
    let workspace = Workspace::with_documents(&["valid.md"]);
    let (status, report) = workspace.check(&["valid.md", "--exclude", "absent/**/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn excluding_every_selected_file_reports_an_operational_error() {
    let workspace = Workspace::with_documents(&["docs/valid.md"]);
    let (status, report) = workspace.check(&["docs", "--exclude", "docs"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn glob_matching_a_directory_does_not_select_its_children() {
    let workspace = Workspace::with_documents(&["docs/service/valid.md"]);
    let (status, report) = workspace.check(&["docs/serv*"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn diagnostics_have_deterministic_order_regardless_of_input_order() {
    let workspace = Workspace::new();
    workspace.write("z.md", "");
    workspace.write("a.md", "");
    let (status, report) = workspace.check(&["z.md", "a.md"]);
    let files: Vec<_> = report["diagnostics"]
        .as_array()
        .expect("diagnostic array")
        .iter()
        .filter_map(|diagnostic| diagnostic["file"].as_str())
        .collect();
    assert_that!(status).is_equal_to(Some(1));
    assert_that!(files.is_empty()).is_equal_to(false);
    assert_that!(files.windows(2).all(|pair| pair[0] <= pair[1])).is_equal_to(true);
}

#[test]
fn parent_traversal_before_a_wildcard_is_normalized() {
    let workspace = Workspace::with_documents(&["docs/sub/kept.md", "docs/direct.md"]);
    let (status, report) = workspace.check(&["docs/sub/../*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn parent_traversal_after_a_wildcard_is_rejected_without_abandoning_valid_files() {
    let workspace = Workspace::with_documents(&["docs/sub/kept.md"]);
    let (status, report) = workspace.check(&["docs/sub/kept.md", "docs/*/../*.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
}

#[cfg(unix)]
#[test]
fn escaped_brackets_match_literal_brackets_in_filenames() {
    let workspace = Workspace::with_documents(&["docs/[ab].md", "docs/a.md"]);
    let (status, report) = workspace.check(&[r"docs/\[ab\].md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn bare_globs_preserve_literal_brackets_in_the_working_directory() {
    let workspace = Workspace::with_documents(&["[docs]/kept.md", "d/skipped.md"]);
    let (status, report) = common::check_in(&workspace.path().join("[docs]"), &["*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn question_mark_matches_one_character_within_a_path_segment() {
    let workspace = Workspace::with_documents(&["docs/a1.md", "docs/a12.md", "docs/a/1.md"]);
    let (status, report) = workspace.check(&["docs/a?.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn brace_alternatives_match_each_requested_filename() {
    let workspace = Workspace::with_documents(&["docs/a.md", "docs/b.md", "docs/c.md"]);
    let (status, report) = workspace.check(&["docs/{a,b}.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn character_classes_match_requested_filename_characters() {
    let workspace = Workspace::with_documents(&["docs/a.md", "docs/b.md", "docs/c.md"]);
    let (status, report) = workspace.check(&["docs/[ab].md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[cfg(unix)]
#[test]
fn explicit_symlink_files_are_deduplicated_by_target() {
    let workspace = Workspace::with_documents(&["target.md"]);
    std::os::unix::fs::symlink("target.md", workspace.path().join("alias.md"))
        .expect("symlink created");
    let (status, report) = workspace.check(&["alias.md", "target.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[cfg(unix)]
#[test]
fn exclusions_apply_to_explicit_symlink_alias_spellings() {
    let workspace = Workspace::with_documents(&["kept.md"]);
    workspace.write("target.md", "");
    std::os::unix::fs::symlink("target.md", workspace.path().join("alias.md"))
        .expect("symlink created");
    let (status, report) = workspace.check(&["alias.md", "kept.md", "--exclude", "alias.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[cfg(unix)]
#[test]
fn directory_globs_do_not_follow_symlinked_subdirectories() {
    let workspace = Workspace::with_documents(&["docs/kept.md", "outside/skipped.md"]);
    std::os::unix::fs::symlink("../outside", workspace.path().join("docs/linked"))
        .expect("symlink created");
    let (status, report) = workspace.check(&["docs/**/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[cfg(unix)]
#[test]
fn explicit_non_utf8_filenames_remain_supported() {
    use std::os::unix::ffi::OsStringExt;
    let workspace = Workspace::new();
    let filename = std::ffi::OsString::from_vec(b"non-utf8-\xff.md".to_vec());
    std::fs::write(workspace.path().join(&filename), common::MINIMAL).expect("fixture written");
    let output = common::binary()
        .current_dir(workspace.path())
        .arg("check")
        .arg(&filename)
        .args(["--format", "json"])
        .output()
        .expect("binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_that!(output.status.code()).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn unmatched_patterns_report_errors_even_when_another_pattern_shares_the_scan_root() {
    let workspace = Workspace::with_documents(&["docs/a.md"]);
    let (status, report) = workspace.check(&["docs/*.md", "docs/missing*.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn absolute_directory_exclusions_apply_to_dot_relative_inputs() {
    let workspace = Workspace::with_documents(&["docs/kept.md", "docs/generated/skipped.md"]);
    let excluded = workspace.path().join("docs/./generated");
    let (status, report) = workspace.check(&[
        "./docs/**/*.md",
        "--exclude",
        excluded.to_str().expect("temporary path is UTF8"),
    ]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn excluded_directory_inputs_do_not_fail_when_other_files_remain() {
    let workspace = Workspace::with_documents(&["docs/kept.md", "generated/skipped.md"]);
    let (status, report) = workspace.check(&["docs", "generated", "--exclude", "generated"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn fully_excluded_glob_matches_do_not_fail_when_other_files_remain() {
    let workspace = Workspace::with_documents(&["docs/kept.md", "generated/skipped.md"]);
    let (status, report) = workspace.check(&["docs", "generated/*.md", "--exclude", "generated"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn explicit_non_markdown_inputs_are_rejected_without_abandoning_valid_files() {
    let workspace = Workspace::with_documents(&["valid.md"]);
    workspace.write("input.txt", "plain text");
    let (status, report) = workspace.check(&["input.txt", "valid.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS902"));
}

#[cfg(unix)]
#[test]
fn socket_inputs_are_rejected_without_abandoning_valid_files() {
    let workspace = Workspace::with_documents(&["valid.md"]);
    let _socket = std::os::unix::net::UnixListener::bind(workspace.path().join("input.md"))
        .expect("socket created");
    let (status, report) = workspace.check(&["input.md", "valid.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS902"));
    assert_that!(report["diagnostics"][0]["message"].as_str())
        .is_equal_to(Some("Expected a regular file or directory"));
}
