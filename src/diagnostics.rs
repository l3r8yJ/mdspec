use crate::model::Position;
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub rule: &'static str,
    pub severity: Severity,
    pub file: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(
        rule: &'static str,
        path: &Path,
        position: Option<Position>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule,
            severity: Severity::Error,
            file: path.to_string_lossy().into_owned(),
            line: position.map(|value| value.line),
            column: position.map(|value| value.column),
            message: message.into(),
            help: None,
        }
    }
}
