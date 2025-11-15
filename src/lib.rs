mod ast;
pub mod cli;
pub mod config;
mod context;
mod engine;
mod lint_set;
pub mod log;
mod output;
mod rule;
mod rules;
mod violation;

use std::io;

pub use config::Config;
pub use config::LintLevel;
pub use engine::LintEngine;
use miette::Diagnostic;
pub use output::{JsonFix, JsonOutput, JsonReplacement, JsonViolation, Summary, format_json};
use thiserror::Error;
use toml::de;
pub use violation::Violation;
pub(crate) use violation::{Fix, Replacement, RuleViolation};

#[derive(Error, Debug, Diagnostic)]
pub(crate) enum LintError {
    #[error("Failed to read file: {0}")]
    #[diagnostic(code(nu_lint::io_error))]
    IoError(#[from] io::Error),

    #[error("Failed to parse configuration: {0}")]
    #[diagnostic(code(nu_lint::config_error))]
    ConfigError(#[from] de::Error),
}
