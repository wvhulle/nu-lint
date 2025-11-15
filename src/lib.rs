mod ast;
pub mod cli;
pub mod config;
mod context;
mod engine;
pub mod fix;
pub mod log;
pub mod output;
mod rule;
mod rules;
mod sets;
mod violation;

use std::io;

pub use config::{Config, LintLevel};
pub use engine::LintEngine;
use miette::Diagnostic;
pub use output::{
    JsonFix, JsonOutput, JsonReplacement, JsonViolation, Summary, VsCodeCodeAction,
    VsCodeDiagnostic, VsCodeJsonOutput, VsCodeLocation, VsCodePosition, VsCodeRange,
    VsCodeRelatedInformation, VsCodeTextEdit, format_json, format_text, format_vscode_json,
};
use thiserror::Error;
use toml::de;
pub use violation::Violation;
pub(crate) use violation::{Fix, Replacement};

#[derive(Error, Debug, Diagnostic)]
pub enum LintError {
    #[error("Failed to read file: {0}")]
    #[diagnostic(code(nu_lint::io_error))]
    IoError(#[from] io::Error),

    #[error("Failed to parse configuration: {0}")]
    #[diagnostic(code(nu_lint::config_error))]
    ConfigError(#[from] de::Error),
}
