use asserting::prelude::*;
use std::process::{Command, Stdio};

#[test]
fn treats_a_closed_output_pipe_as_success() {
    let (reader, writer) = std::io::pipe().expect("pipe is created");
    drop(reader);
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
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
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
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
    let directory = tempfile::tempdir().expect("temporary directory created");
    let nested = directory.path().join("nested.md");
    std::fs::create_dir(&nested).expect("nested directory created");
    std::fs::write(
        nested.join("endpoint.MD"),
        include_str!("fixtures/minimal.md"),
    )
    .expect("valid document written");
    std::fs::write(directory.path().join("notes.txt"), "Not an endpoint")
        .expect("non-Markdown file written");
    let output = Command::new(env!("CARGO_BIN_EXE_mdspec"))
        .arg("check")
        .arg(directory.path())
        .args(["--format", "json"])
        .output()
        .expect("the built binary starts");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_that!(output.status.code()).is_equal_to(Some(0));
    assert_that!(report["files_checked"].as_u64()).is_equal_to(Some(1));
    assert_that!(report["error_count"].as_u64()).is_equal_to(Some(0));
}
