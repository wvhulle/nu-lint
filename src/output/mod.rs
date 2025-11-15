mod json;
mod text;
mod vscode;

use std::{borrow::Cow, fs};

pub use json::{JsonFix, JsonOutput, JsonReplacement, JsonViolation, format_json};
use serde::Serialize;
pub use text::format_text;
pub use vscode::{
    VsCodeCodeAction, VsCodeDiagnostic, VsCodeJsonOutput, VsCodeLocation, VsCodePosition,
    VsCodeRange, VsCodeRelatedInformation, VsCodeTextEdit, format_vscode_json,
};

use crate::{config::LintLevel, violation::Violation};

#[derive(Serialize)]
pub struct Summary {
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub files_checked: usize,
}

impl Summary {
    #[must_use]
    pub fn from_violations(violations: &[Violation]) -> Self {
        let (errors, warnings, info) = violations.iter().fold(
            (0, 0, 0),
            |(errors, warnings, info), violation| match violation.lint_level {
                LintLevel::Deny => (errors + 1, warnings, info),
                LintLevel::Warn => (errors, warnings + 1, info),
                LintLevel::Allow => (errors, warnings, info + 1),
            },
        );

        Self {
            errors,
            warnings,
            info,
            files_checked: 1,
        }
    }

    #[must_use]
    pub fn format_compact(&self) -> String {
        let parts: Vec<String> = [
            (self.errors > 0).then(|| format!("{} error(s)", self.errors)),
            (self.warnings > 0).then(|| format!("{} warning(s)", self.warnings)),
            (self.info > 0).then(|| format!("{} info", self.info)),
        ]
        .into_iter()
        .flatten()
        .collect();

        if parts.is_empty() {
            String::from("0 violations")
        } else {
            parts.join(", ")
        }
    }
}

pub(super) fn calculate_line_column(source: &str, offset: usize) -> (usize, usize) {
    source
        .char_indices()
        .take_while(|(pos, _)| *pos < offset)
        .fold((1, 1), |(line, column), (_, ch)| {
            if ch == '\n' {
                (line + 1, 1)
            } else {
                (line, column + 1)
            }
        })
}

pub(super) fn read_source_code(file: Option<&Cow<'_, str>>) -> String {
    file.and_then(|path| fs::read_to_string(path.as_ref()).ok())
        .unwrap_or_default()
}
