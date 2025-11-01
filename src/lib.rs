pub mod ast;
pub mod cli;
pub mod config;
pub mod context;
pub mod engine;
pub mod external_command;
pub mod log;
pub mod output;
pub mod rule;
pub mod rules;
pub mod violation;

pub use config::Config;
pub use context::LintContext;
pub use engine::LintEngine;
use miette::Diagnostic;
pub use output::{JsonFormatter, JsonOutput, OutputFormat, OutputFormatter, TextFormatter};
pub use rule::{Rule, RuleCategory};
use thiserror::Error;
pub use violation::{Fix, Replacement, RuleViolation, Severity, Violation};

#[derive(Error, Debug, Diagnostic)]
pub enum LintError {
    #[error("Failed to read file: {0}")]
    #[diagnostic(code(nu_lint::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse configuration: {0}")]
    #[diagnostic(code(nu_lint::config_error))]
    ConfigError(#[from] toml::de::Error),
}
