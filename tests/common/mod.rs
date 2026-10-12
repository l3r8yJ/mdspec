#![allow(dead_code)]

use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

pub const MINIMAL: &str = include_str!("../fixtures/minimal.md");

pub fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mdspec"))
}

pub fn output(arguments: &[&str]) -> Output {
    binary().args(arguments).output().expect("binary starts")
}

pub fn parse(output: &Output) -> (Option<i32>, Value) {
    let report = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    (output.status.code(), report)
}

pub fn report(arguments: &[&str]) -> (Option<i32>, Value) {
    parse(&output(arguments))
}

pub fn output_in(directory: &Path, arguments: &[&str]) -> Output {
    binary()
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("binary starts")
}

pub fn check_in(directory: &Path, arguments: &[&str]) -> (Option<i32>, Value) {
    let arguments = [&["check"], arguments, &["--format", "json"]].concat();
    parse(&output_in(directory, &arguments))
}

pub fn diagnostics(report: &Value) -> &[Value] {
    report["diagnostics"]
        .as_array()
        .expect("diagnostics is an array")
}

pub fn rules(report: &Value) -> Vec<&str> {
    diagnostics(report)
        .iter()
        .filter_map(|diagnostic| diagnostic["rule"].as_str())
        .collect()
}

pub fn rule_names(report: &Value) -> Vec<String> {
    rules(report).into_iter().map(str::to_owned).collect()
}

pub fn rules_with_lines(report: &Value) -> Vec<(String, u64)> {
    diagnostics(report)
        .iter()
        .map(|diagnostic| {
            (
                diagnostic["rule"].as_str().expect("rule string").to_owned(),
                diagnostic["line"].as_u64().unwrap_or_default(),
            )
        })
        .collect()
}

pub fn rule(name: &str, line: u64) -> (String, u64) {
    (name.to_owned(), line)
}

pub fn rule_count(report: &Value, rule: &str) -> usize {
    diagnostics(report)
        .iter()
        .filter(|diagnostic| diagnostic["rule"] == rule)
        .count()
}

pub fn exit_and_rule_names(path: &str) -> (i32, Vec<String>) {
    let (status, report) = report(&["check", path, "--format", "json"]);
    (status.expect("exit code"), rule_names(&report))
}

pub fn check_strict(document: &str, config: &str) -> (Option<i32>, Value) {
    report(&[
        "check", document, "--config", config, "--strict", "--format", "json",
    ])
}

pub fn check_fixture(file: &str, strict: bool) -> (Option<i32>, Value) {
    let mut arguments = vec!["check", file, "--format", "json"];
    if strict {
        arguments.push("--strict");
    }
    report(&arguments)
}

pub fn config_check(contents: &str) -> (Option<i32>, Value) {
    let workspace = Workspace::new();
    let config = workspace.config(contents);
    check_strict("tests/fixtures/minimal.md", &config)
}

pub fn label_error(contents: &str) -> (Option<i32>, String) {
    let (code, report) = config_check(contents);
    let message = report["diagnostics"][0]["message"]
        .as_str()
        .expect("the diagnostic carries a message")
        .to_owned();
    (code, message)
}

pub struct Workspace {
    directory: tempfile::TempDir,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            directory: tempfile::tempdir().expect("temporary directory created"),
        }
    }

    pub fn with_documents(documents: &[&str]) -> Self {
        let workspace = Self::new();
        for document in documents {
            workspace.document(document);
        }
        workspace
    }

    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    pub fn write(&self, relative: &str, contents: impl AsRef<[u8]>) -> &Self {
        let path = self.path().join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture has a parent"))
            .expect("fixture directory created");
        std::fs::write(path, contents).expect("fixture written");
        self
    }

    pub fn document(&self, relative: &str) -> &Self {
        self.write(relative, MINIMAL)
    }

    pub fn config(&self, contents: &str) -> String {
        self.write("mdspec.toml", contents);
        self.path()
            .join("mdspec.toml")
            .to_str()
            .expect("temporary path is UTF8")
            .to_owned()
    }

    pub fn output(&self, arguments: &[&str]) -> Output {
        output_in(self.path(), arguments)
    }

    pub fn check(&self, arguments: &[&str]) -> (Option<i32>, Value) {
        check_in(self.path(), arguments)
    }

    pub fn check_default(&self, document: &str) -> (Option<i32>, Value) {
        self.check_document(document, "", false)
    }

    pub fn check_document(
        &self,
        document: &str,
        config: &str,
        strict: bool,
    ) -> (Option<i32>, Value) {
        self.write("endpoint.md", document);
        let config = self.config(config);
        let mut arguments = vec!["endpoint.md", "--config", config.as_str()];
        if strict {
            arguments.push("--strict");
        }
        self.check(&arguments)
    }

    pub fn check_rules(&self, document: &str, config: &str, strict: bool) -> Vec<(String, u64)> {
        let (_, report) = self.check_document(document, config, strict);
        rules_with_lines(&report)
    }

    pub fn check_rule_names(&self, document: &str, config: &str, strict: bool) -> Vec<String> {
        let (_, report) = self.check_document(document, config, strict);
        rule_names(&report)
    }
}
