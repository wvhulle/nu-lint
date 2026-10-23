mod ast;
pub mod cli;
mod config;
mod config_loading;
mod context;
mod dsl;
mod effect;
mod engine;
mod file_collection;
mod fix;
mod format;
mod ignore;
pub mod log;
#[cfg(feature = "lsp")]
mod lsp;
mod rule;
mod rules;
mod span;
mod violation;

use std::{error::Error, fmt, io, path::PathBuf};

pub use config::{Config, LintLevel};
pub use engine::LintEngine;
pub use file_collection::collect_nu_files_from_dir;
pub use fix::apply_fixes_iteratively;
use rule::RuleCatalog;
use toml::{de, ser};
use violation::{Fix, Replacement};

pub const NU_PARSER_VERSION: &str = env!("NU_PARSER_VERSION");

pub const RULE_CATALOG: RuleCatalog = RuleCatalog {
    rules: rules::USED_RULES,
    groups: rules::groups::ALL_GROUPS,
};

pub fn all_rule_ids() -> impl Iterator<Item = &'static str> {
    RULE_CATALOG.rules.iter().map(|rule| rule.id())
}

#[must_use]
pub fn lint_engine_with_all_rules(config: Config) -> LintEngine {
    LintEngine::new(config, RULE_CATALOG)
}

#[derive(Debug)]
pub enum LintError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Config {
        source: de::Error,
    },
    ConfigSerialize {
        source: ser::Error,
    },
    RuleDoesNotExist {
        non_existing_id: String,
    },
    RuleConflict {
        rule_a: &'static str,
        rule_b: &'static str,
    },
    NoConfigLocation,
}

impl fmt::Display for LintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "failed to read '{}': {source}", path.display())
            }
            Self::RuleDoesNotExist { non_existing_id } => write!(
                f,
                "Rule declared in config with id `{non_existing_id}` does not exist in this \
                 version."
            ),
            Self::Config { source } => write!(f, "invalid configuration: {source}"),
            Self::ConfigSerialize { source } => {
                write!(f, "failed to serialize configuration: {source}")
            }
            Self::RuleConflict { rule_a, rule_b } => {
                write!(
                    f,
                    "Based on the defaults merged with any configuration file (if present), the \
                     following two rules are both enabled and conclicting: '{rule_a}' and \
                     '{rule_b}'. The linter will not be able to start until you make sure at most \
                     one of both is active. Use the configuration file to override the lint \
                     levels."
                )
            }
            Self::NoConfigLocation => {
                write!(f, "no workspace root or home directory available")
            }
        }
    }
}

impl Error for LintError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Config { source } => Some(source),
            Self::ConfigSerialize { source } => Some(source),
            Self::RuleConflict { .. } | Self::RuleDoesNotExist { .. } | Self::NoConfigLocation => {
                None
            }
        }
    }
}
