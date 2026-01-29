mod text;

use std::fs;

use miette::Severity;
use serde::Serialize;
pub use text::format_text;

use crate::violation::{SourceFile, Violation};

/// Output format for linting results
#[derive(clap::ValueEnum, Clone, Copy, Default)]
pub enum Format {
    /// Human-readable text format (default)
    #[default]
    Text,
}

/// Format and output linting results
#[must_use]
pub fn format_output(violations: &[Violation], format: Format) -> String {
    match format {
        Format::Text => format_text(violations),
    }
}

#[derive(Serialize)]
pub struct Summary {
    pub errors: usize,
    pub warnings: usize,
    pub hints: usize,
    pub files_checked: usize,
}

impl Summary {
    #[must_use]
    pub fn from_violations(violations: &[Violation]) -> Self {
        let (errors, warnings, hints) = violations.iter().fold(
            (0, 0, 0),
            |(errors, warnings, hints), violation| match violation.lint_level {
                Severity::Error => (errors + 1, warnings, hints),
                Severity::Warning => (errors, warnings + 1, hints),
                Severity::Advice => (errors, warnings, hints + 1),
            },
        );

        Self {
            errors,
            warnings,
            hints,
            files_checked: 1,
        }
    }

    #[must_use]
    pub fn format_compact(&self) -> String {
        let parts: Vec<String> = [
            (self.errors > 0).then(|| format!("{} error(s)", self.errors)),
            (self.warnings > 0).then(|| format!("{} warning(s)", self.warnings)),
            (self.hints > 0).then(|| format!("{} hint(s)", self.hints)),
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

pub fn read_source_code(file: Option<&SourceFile>) -> String {
    file.and_then(|f| f.as_path())
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_default()
}
