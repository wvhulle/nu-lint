#![allow(
    clippy::excessive_nesting,
    reason = "AST traversal naturally requires nested pattern matching"
)]

mod ast;
pub mod cli;
mod config;
mod context;
mod effect;
mod engine;
mod external_commands;
mod fix;
mod log;
mod lsp;
mod output;
mod rule;
mod rules;
mod span;
mod violation;

use std::{error::Error, fmt, io, path::PathBuf};

pub use config::{Config, LintLevel};
pub use engine::LintEngine;
pub use fix::apply_fixes_iteratively;
use toml::de;
use violation::{Fix, Replacement};

const NU_PARSER_VERSION: &str = env!("NU_PARSER_VERSION");

#[derive(Debug)]
enum LintError {
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
