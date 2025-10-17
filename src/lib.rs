pub mod cli;
pub mod config;
pub mod context;
pub mod engine;
pub mod external_command;
pub mod lint;
pub mod output;
pub mod rule;
pub mod rules;
pub mod visitor;

pub use config::Config;
pub use context::LintContext;
pub use engine::LintEngine;
pub use lint::{Fix, Replacement, Severity, Violation};
use miette::Diagnostic;
pub use output::{JsonFormatter, JsonOutput, OutputFormat, OutputFormatter, TextFormatter};
pub use rule::{Rule, RuleCategory};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum LintError {
    #[error("Failed to read file: {0}")]
    #[diagnostic(code(nu_lint::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse configuration: {0}")]
    #[diagnostic(code(nu_lint::config_error))]
    ConfigError(#[from] toml::de::Error),
}
