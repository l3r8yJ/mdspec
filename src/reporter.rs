use crate::cli::Format;
use crate::diagnostics::{Diagnostic, Severity};
use serde::Serialize;
use std::io::{self, Write};

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
            (&left.file, left.line, left.column, left.rule, &left.message).cmp(&(
                &right.file,
                right.line,
                right.column,
                right.rule,
                &right.message,
            ))
        });
        Self {
            schema_version: 1,
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
            .any(|item| matches!(item.rule, "MDS900" | "MDS901" | "MDS902"))
        {
            2
        } else {
            u8::from(self.error_count > 0)
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
