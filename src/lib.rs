mod alternatives;
mod ast;
pub mod cli;
pub mod config;
mod context;
mod effect;
mod engine;
pub mod fix;
pub mod log;
pub mod lsp;
pub mod output;
mod rule;
mod rules;
mod violation;

use std::{error::Error, fmt, io, path::PathBuf};

pub use config::{Config, LintLevel};
pub use engine::LintEngine;
pub use output::{
    JsonFix, JsonOutput, JsonReplacement, JsonViolation, Summary, format_json, format_text,
};
use toml::de;
pub use violation::Violation;
pub(crate) use violation::{Fix, Replacement};

#[derive(Debug)]
pub enum LintError {
    Io { path: PathBuf, source: io::Error },
    Config { source: de::Error },
}

impl fmt::Display for LintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "failed to read '{}': {source}", path.display())
            }
            Self::Config { source } => write!(f, "invalid configuration: {source}"),
        }
    }
}

impl Error for LintError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Config { source } => Some(source),
        }
    }
}
