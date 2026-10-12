#![cfg(test)]

use asserting::prelude::*;
use std::path::Path;
use std::process::Command;

fn document(directory: &Path, relative: &str) {
    let path = directory.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture has parent"))
        .expect("fixture directory created");
    std::fs::write(path, include_str!("fixtures/minimal.md")).expect("fixture written");
}

fn check(directory: &Path, arguments: &[&str]) -> (Option<i32>, serde_json::Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .current_dir(directory)
        .arg("check")
        .args(arguments)
        .args(["--format", "json"])
        .output()
        .expect("binary starts");
    let report = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    (output.status.code(), report)
}

#[test]
fn checks_multiple_explicit_files_in_one_invocation() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "first.md");
    document(directory.path(), "second.md");
    let (status, report) = check(directory.path(), &["first.md", "second.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn single_star_does_not_cross_directory_boundaries() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/api/service/one.md");
    document(directory.path(), "docs/team/api/service/two.md");
    document(directory.path(), "docs/api/service/nested/three.md");
    let (status, report) = check(directory.path(), &["docs/*/service/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn recursive_glob_handles_zero_or_multiple_directory_segments() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/service/direct.md");
    document(directory.path(), "docs/team/api/service/nested/deep.md");
    let (status, report) = check(directory.path(), &["docs/**/service/**/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn exclusions_win_over_all_inputs_and_skip_matching_directory_contents() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/kept.md");
    document(directory.path(), "docs/generated/nested/skipped.md");
    document(directory.path(), "docs/private.md");
    let (status, report) = check(
        directory.path(),
        &[
            "docs",
            "docs/private.md",
            "--exclude",
            "docs/generated",
            "--exclude",
            "docs/private.md",
        ],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn exclusion_globs_use_the_same_segment_and_recursive_matching() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/a/service/kept.md");
    document(directory.path(), "docs/a/service/generated/deep.md");
    document(directory.path(), "docs/a/service/old.md");
    let (status, report) = check(
        directory.path(),
        &[
            "docs/**/*.md",
            "--exclude",
            "docs/**/generated",
            "--exclude",
            "docs/*/service/old.md",
        ],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn directory_and_glob_discovery_respect_ignore_and_hidden_files() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/kept.md");
    document(directory.path(), "docs/ignored.md");
    document(directory.path(), "docs/.hidden.md");
    std::fs::write(directory.path().join(".gitignore"), "ignored.md\n")
        .expect("ignore file written");
    let (status, report) = check(directory.path(), &["docs", "docs/**/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn explicit_files_bypass_ignore_and_hidden_discovery_rules() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "ignored.md");
    document(directory.path(), ".hidden.md");
    std::fs::write(directory.path().join(".ignore"), "ignored.md\n").expect("ignore file written");
    let (status, report) = check(directory.path(), &["ignored.md", ".hidden.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn overlapping_inputs_and_normalized_paths_validate_each_file_once() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/first.md");
    document(directory.path(), "docs/sub/second.md");
    let absolute = directory.path().join("docs/first.md");
    let (status, report) = check(
        directory.path(),
        &[
            "docs",
            "./docs/**/*.md",
            "docs/sub/../first.md",
            absolute.to_str().expect("temporary path is UTF8"),
        ],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn existing_literal_paths_with_brackets_take_precedence_over_globs() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/[ab].md");
    document(directory.path(), "docs/a.md");
    let (status, report) = check(directory.path(), &["docs/[ab].md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn absolute_globs_and_relative_exclusions_share_path_identity() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "доки с пробелом/первый.md");
    document(directory.path(), "доки с пробелом/второй.md");
    let pattern = directory.path().join("доки с пробелом/*.md");
    let (status, report) = check(
        directory.path(),
        &[
            pattern.to_str().expect("temporary path is UTF8"),
            "--exclude",
            "./доки с пробелом/первый.md",
        ],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn invalid_and_unmatched_inputs_do_not_abandon_valid_documents() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    std::fs::write(directory.path().join("empty.md"), "").expect("empty fixture written");
    let (status, report) = check(
        directory.path(),
        &["empty.md", "missing.md", "missing/**/*.md", "docs/["],
    );
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
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "valid.md");
    let (status, report) = check(directory.path(), &["valid.md", "--exclude", "docs/["]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn unmatched_exclusions_are_valid() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "valid.md");
    let (status, report) = check(
        directory.path(),
        &["valid.md", "--exclude", "absent/**/*.md"],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn excluding_every_selected_file_reports_an_operational_error() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/valid.md");
    let (status, report) = check(directory.path(), &["docs", "--exclude", "docs"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(0));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn glob_matching_a_directory_does_not_select_its_children() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/service/valid.md");
    let (status, report) = check(directory.path(), &["docs/serv*"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn diagnostics_have_deterministic_order_regardless_of_input_order() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    std::fs::write(directory.path().join("z.md"), "").expect("empty fixture written");
    std::fs::write(directory.path().join("a.md"), "").expect("empty fixture written");
    let (status, report) = check(directory.path(), &["z.md", "a.md"]);
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
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/sub/kept.md");
    document(directory.path(), "docs/direct.md");
    let (status, report) = check(directory.path(), &["docs/sub/../*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn parent_traversal_after_a_wildcard_is_rejected_without_abandoning_valid_files() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/sub/kept.md");
    let (status, report) = check(directory.path(), &["docs/sub/kept.md", "docs/*/../*.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
}

#[cfg(unix)]
#[test]
fn escaped_brackets_match_literal_brackets_in_filenames() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/[ab].md");
    document(directory.path(), "docs/a.md");
    let (status, report) = check(directory.path(), &[r"docs/\[ab\].md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn bare_globs_preserve_literal_brackets_in_the_working_directory() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "[docs]/kept.md");
    document(directory.path(), "d/skipped.md");
    let (status, report) = check(&directory.path().join("[docs]"), &["*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn question_mark_matches_one_character_within_a_path_segment() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/a1.md");
    document(directory.path(), "docs/a12.md");
    document(directory.path(), "docs/a/1.md");
    let (status, report) = check(directory.path(), &["docs/a?.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn brace_alternatives_match_each_requested_filename() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/a.md");
    document(directory.path(), "docs/b.md");
    document(directory.path(), "docs/c.md");
    let (status, report) = check(directory.path(), &["docs/{a,b}.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[test]
fn character_classes_match_requested_filename_characters() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/a.md");
    document(directory.path(), "docs/b.md");
    document(directory.path(), "docs/c.md");
    let (status, report) = check(directory.path(), &["docs/[ab].md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(2));
}

#[cfg(unix)]
#[test]
fn explicit_symlink_files_are_deduplicated_by_target() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "target.md");
    std::os::unix::fs::symlink("target.md", directory.path().join("alias.md"))
        .expect("symlink created");
    let (status, report) = check(directory.path(), &["alias.md", "target.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[cfg(unix)]
#[test]
fn exclusions_apply_to_explicit_symlink_alias_spellings() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "kept.md");
    std::fs::write(directory.path().join("target.md"), "").expect("empty fixture written");
    std::os::unix::fs::symlink("target.md", directory.path().join("alias.md"))
        .expect("symlink created");
    let (status, report) = check(
        directory.path(),
        &["alias.md", "kept.md", "--exclude", "alias.md"],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[cfg(unix)]
#[test]
fn directory_globs_do_not_follow_symlinked_subdirectories() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/kept.md");
    document(directory.path(), "outside/skipped.md");
    std::os::unix::fs::symlink("../outside", directory.path().join("docs/linked"))
        .expect("symlink created");
    let (status, report) = check(directory.path(), &["docs/**/*.md"]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[cfg(unix)]
#[test]
fn explicit_non_utf8_filenames_remain_supported() {
    use std::os::unix::ffi::OsStringExt;
    let directory = tempfile::tempdir().expect("temporary directory created");
    let filename = std::ffi::OsString::from_vec(b"non-utf8-\xff.md".to_vec());
    std::fs::write(
        directory.path().join(&filename),
        include_str!("fixtures/minimal.md"),
    )
    .expect("fixture written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .current_dir(directory.path())
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
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/a.md");
    let (status, report) = check(directory.path(), &["docs/*.md", "docs/missing*.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn absolute_directory_exclusions_apply_to_dot_relative_inputs() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/kept.md");
    document(directory.path(), "docs/generated/skipped.md");
    let excluded = directory.path().join("docs/./generated");
    let (status, report) = check(
        directory.path(),
        &[
            "./docs/**/*.md",
            "--exclude",
            excluded.to_str().expect("temporary path is UTF8"),
        ],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
}

#[test]
fn excluded_directory_inputs_do_not_fail_when_other_files_remain() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/kept.md");
    document(directory.path(), "generated/skipped.md");
    let (status, report) = check(
        directory.path(),
        &["docs", "generated", "--exclude", "generated"],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn fully_excluded_glob_matches_do_not_fail_when_other_files_remain() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "docs/kept.md");
    document(directory.path(), "generated/skipped.md");
    let (status, report) = check(
        directory.path(),
        &["docs", "generated/*.md", "--exclude", "generated"],
    );
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}

#[test]
fn explicit_non_markdown_inputs_are_rejected_without_abandoning_valid_files() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "valid.md");
    std::fs::write(directory.path().join("input.txt"), "plain text")
        .expect("non Markdown fixture written");
    let (status, report) = check(directory.path(), &["input.txt", "valid.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS902"));
}

#[cfg(unix)]
#[test]
fn socket_inputs_are_rejected_without_abandoning_valid_files() {
    let directory = tempfile::tempdir().expect("temporary directory created");
    document(directory.path(), "valid.md");
    let _socket = std::os::unix::net::UnixListener::bind(directory.path().join("input.md"))
        .expect("socket created");
    let (status, report) = check(directory.path(), &["input.md", "valid.md"]);
    assert_that!(status).is_equal_to(Some(2));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["diagnostics"][0]["rule"].as_str()).is_equal_to(Some("MDS902"));
    assert_that!(report["diagnostics"][0]["message"].as_str())
        .is_equal_to(Some("Expected a regular file or directory"));
}
