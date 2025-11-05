mod ast;
pub mod cli;
pub mod config;
mod context;
mod engine;
mod external_command;
pub mod log;
mod output;
mod rule;
mod rules;
mod violation;
pub use config::Config;
pub use engine::LintEngine;
use miette::Diagnostic;
pub use output::{JsonFix, JsonOutput, JsonReplacement, JsonViolation, Summary, format_json};
use thiserror::Error;
pub(crate) use violation::{Fix, Replacement, RuleViolation};
pub use violation::{Severity, Violation};

#[derive(Error, Debug, Diagnostic)]
pub enum LintError {
    #[error("Failed to read file: {0}")]
    #[diagnostic(code(nu_lint::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse configuration: {0}")]
    #[diagnostic(code(nu_lint::config_error))]
    ConfigError(#[from] toml::de::Error),
}
