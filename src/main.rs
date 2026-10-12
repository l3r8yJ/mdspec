mod cli;
mod config;
mod diagnostics;
mod model;
mod parser;
mod references;
mod reporter;
mod rules;
mod workspace;

use clap::Parser;
use cli::{Cli, Command};
use config::Config;
use diagnostics::{Diagnostic, Severity};
use reporter::Report;
use std::fs;
use std::io;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Cli {
        command:
            Command::Check {
                inputs,
                exclude,
                strict,
                format,
                config,
            },
    } = Cli::parse();
    let report = check(&inputs, &exclude, config.as_deref(), strict);
    match reporter::print(&report, format) {
        Ok(()) => ExitCode::from(report.exit_code()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Could not write report: {error}");
            ExitCode::from(2)
        }
    }
}

fn check(
    inputs: &[std::path::PathBuf],
    exclude: &[String],
    config_path: Option<&Path>,
    strict: bool,
) -> Report {
    let config = match load_config(config_path) {
        Ok(config) => config,
        Err(error) => return Report::new(0, vec![*error]),
    };
    let (files, mut diagnostics) = workspace::discover(inputs, exclude);
    let mut files_checked = 0;
    for file in files {
        match fs::read_to_string(&file) {
            Ok(source) => {
                let document = parser::parse(&source);
                diagnostics.extend(rules::validate(&document, &file, &config));
                diagnostics.extend(references::validate(&document, &file, &config, strict));
                files_checked += 1;
            }
            Err(error) => diagnostics.push(Diagnostic::error(
                "MDS901",
                &file,
                None,
                format!("Could not read UTF-8 document: {error}"),
            )),
        }
    }
    if strict || config.diagnostics.warnings_as_errors {
        for diagnostic in &mut diagnostics {
            diagnostic.severity = Severity::Error;
        }
    }
    Report::new(files_checked, diagnostics)
}

fn load_config(path: Option<&Path>) -> Result<Config, Box<Diagnostic>> {
    let Some(path) = path else {
        return Ok(Config::default());
    };
    let source = fs::read_to_string(path).map_err(|error| {
        Box::new(Diagnostic::error(
            "MDS900",
            path,
            None,
            format!("Could not read configuration: {error}"),
        ))
    })?;
    let config: Config = toml::from_str(&source).map_err(|error| {
        Box::new(Diagnostic::error(
            "MDS900",
            path,
            None,
            format!("Invalid TOML configuration: {error}"),
        ))
    })?;
    if !config.labels.is_valid() {
        return Err(Box::new(Diagnostic::error(
            "MDS900",
            path,
            None,
            "Labels must be nonempty single-line strings without surrounding whitespace; section labels must be distinct, and source and target labels must differ",
        )));
    }
    Ok(config)
}
