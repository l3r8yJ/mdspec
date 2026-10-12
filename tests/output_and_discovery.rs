#![cfg(test)]

mod common;

use asserting::prelude::*;
use common::Workspace;
use std::process::Stdio;

#[test]
fn treats_a_closed_output_pipe_as_success() {
    let (reader, writer) = std::io::pipe().expect("pipe is created");
    drop(reader);
    let output = common::binary()
        .args(["check", "tests/fixtures/empty.md", "--format", "json"])
        .stdout(Stdio::from(writer))
        .output()
        .expect("the built binary starts");
    assert_that!(output.status.code()).is_equal_to(Some(0));
    assert_that!(output.stderr.is_empty()).is_equal_to(true);
}

#[cfg(target_os = "linux")]
#[test]
fn reports_output_device_failure_as_an_operational_error() {
    let output_device = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("Linux full device opens");
    let output = common::binary()
        .args(["check", "tests/fixtures/empty.md", "--format", "json"])
        .stdout(Stdio::from(output_device))
        .output()
        .expect("the built binary starts");
    assert_that!(output.status.code()).is_equal_to(Some(2));
    assert_that!(String::from_utf8_lossy(&output.stderr).contains("Could not write report:"))
        .is_equal_to(true);
}

#[test]
fn discovers_only_markdown_files_even_when_a_directory_has_a_markdown_extension() {
    let workspace = Workspace::new();
    workspace.write("nested.md/endpoint.MD", include_str!("fixtures/minimal.md"));
    workspace.write("notes.txt", "Not an endpoint");
    let (status, report) = workspace.check(&["."]);
    assert_that!(status).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}
