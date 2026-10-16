mod cli;
mod config;
mod diagnostics;
mod links;
mod model;
mod outline;
mod parser;
mod reporter;
mod rule;
mod rules;
mod workspace;

use clap::Parser;
use cli::{Cli, Command};
use config::Config;
use diagnostics::{Diagnostic, Severity};
use reporter::Report;
use rule::Rule;
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
            ExitCode::from(reporter::exit::OPERATIONAL_ERROR)
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
    let mut documents = Vec::new();
    for file in files {
        match fs::read_to_string(&file) {
            Ok(source) => documents.push((file, parser::parse(&source))),
            Err(error) => diagnostics.push(Diagnostic::error(
                Rule::Mds901,
                &file,
                None,
                format!("Could not read UTF-8 document: {error}"),
            )),
        }
    }
    let sources: Vec<_> = documents
        .iter()
        .map(|(file, document)| rules::Source {
            path: file,
            document,
            outline: outline::build(document, &config.labels),
        })
        .collect();
    diagnostics.extend(rules::check(&sources, &config));
    if strict || config.diagnostics.warnings_as_errors {
        for diagnostic in &mut diagnostics {
            diagnostic.severity = Severity::Error;
        }
    }
    Report::new(documents.len(), diagnostics)
}

fn load_config(path: Option<&Path>) -> Result<Config, Box<Diagnostic>> {
    let Some(path) = path else {
        return Ok(Config::default());
    };
    let source = fs::read_to_string(path).map_err(|error| {
        Box::new(Diagnostic::error(
            Rule::Mds900,
            path,
            None,
            format!("Could not read configuration: {error}"),
        ))
    })?;
    let config: Config = toml::from_str(&source).map_err(|error| {
        Box::new(Diagnostic::error(
            Rule::Mds900,
            path,
            None,
            format!("Invalid TOML configuration: {error}"),
        ))
    })?;
    let label_errors = config.labels.validation_errors();
    if !label_errors.is_empty() {
        return Err(Box::new(Diagnostic::error(
            Rule::Mds900,
            path,
            None,
            label_errors.join("; "),
        )));
    }
    Ok(config)
}
