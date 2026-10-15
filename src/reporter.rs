use crate::cli::Format;
use crate::diagnostics::{Diagnostic, Severity};
use serde::Serialize;
use std::io::{self, Write};

const SCHEMA_VERSION: u8 = 1;

pub mod exit {
    pub const CLEAN: u8 = 0;
    pub const VIOLATIONS: u8 = 1;
    pub const OPERATIONAL_ERROR: u8 = 2;
}

#[derive(Serialize)]
pub struct Report {
    pub schema_version: u8,
    pub files_checked: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl Report {
    pub fn new(files_checked: usize, mut diagnostics: Vec<Diagnostic>) -> Self {
        diagnostics.sort_by(|left, right| {
            (
                &left.file,
                left.line,
                left.column,
                left.rule.as_str(),
                &left.message,
            )
                .cmp(&(
                    &right.file,
                    right.line,
                    right.column,
                    right.rule.as_str(),
                    &right.message,
                ))
        });
        Self {
            schema_version: SCHEMA_VERSION,
            files_checked,
            error_count: diagnostics
                .iter()
                .filter(|item| item.severity == Severity::Error)
                .count(),
            warning_count: diagnostics
                .iter()
                .filter(|item| item.severity == Severity::Warning)
                .count(),
            diagnostics,
        }
    }
    pub fn exit_code(&self) -> u8 {
        if self
            .diagnostics
            .iter()
            .any(|item| item.rule.is_operational())
        {
            exit::OPERATIONAL_ERROR
        } else if self.error_count > 0 {
            exit::VIOLATIONS
        } else {
            exit::CLEAN
        }
    }
}

pub fn print(report: &Report, format: Format) -> io::Result<()> {
    match format {
        Format::Json => {
            let mut output = io::BufWriter::new(io::stdout().lock());
            serde_json::to_writer_pretty(&mut output, report)?;
            writeln!(output)?;
            output.flush()
        }
        Format::Text => {
            let mut output = io::BufWriter::new(io::stderr().lock());
            for diagnostic in &report.diagnostics {
                write!(output, "{}", diagnostic.file)?;
                if let (Some(line), Some(column)) = (diagnostic.line, diagnostic.column) {
                    write!(output, ":{line}:{column}")?;
                }
                writeln!(output)?;
                let severity = match diagnostic.severity {
                    Severity::Error => "ERROR",
                    Severity::Warning => "WARNING",
                };
                writeln!(
                    output,
                    "{severity} {}: {}",
                    diagnostic.rule, diagnostic.message
                )?;
                if let Some(help) = &diagnostic.help {
                    writeln!(output, "help: {help}")?;
                }
                writeln!(output)?;
            }
            writeln!(
                output,
                "Files checked: {}; errors: {}; warnings: {}",
                report.files_checked, report.error_count, report.warning_count
            )?;
            output.flush()
        }
    }
}
