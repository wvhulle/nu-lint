use core::fmt::{self, Display};
use std::{
    collections::HashMap,
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    process,
};

use serde::{Deserialize, Serialize};

use crate::lint_set::LintSet;

/// Lint level configuration (inspired by Clippy)
/// - Allow: Don't report this lint
/// - Warn: Report as a warning
/// - Deny: Report as an error
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    Allow,
    #[default]
    Warn,
    Deny,
}

impl Display for LintLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Warn => write!(f, "warn"),
            Self::Deny => write!(f, "deny"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub lints: LintConfig,

    #[serde(default)]
    pub style: StyleConfig,

    #[serde(default)]
    pub exclude: ExcludeConfig,

    #[serde(default)]
    pub fix: FixConfig,
}

/// Lint configuration with support for set-level and individual rule
/// configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct LintConfig {
    /// Configure entire lint sets (e.g., "naming", "idioms", "pedantic")
    #[serde(default)]
    pub sets: HashMap<LintSet, LintLevel>,

    /// Configure individual rules (overrides set settings)
    #[serde(default)]
    pub rules: HashMap<String, LintLevel>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct StyleConfig {
    #[serde(default = "StyleConfig::default_line_length")]
    pub line_length: usize,

    #[serde(default = "StyleConfig::default_indent_spaces")]
    pub indent_spaces: usize,
}

impl StyleConfig {
    const fn default_line_length() -> usize {
        100
    }

    const fn default_indent_spaces() -> usize {
        4
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct ExcludeConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct FixConfig {
    pub enabled: bool,

    pub safe_only: bool,
}

impl Config {
    /// Load configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the TOML content is
    /// invalid.
    pub(crate) fn load_from_file(path: &Path) -> Result<Self, crate::LintError> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Load configuration from file or use defaults
    #[must_use]
    pub fn load(config_path: Option<&PathBuf>) -> Self {
        config_path
            .cloned()
            .or_else(find_config_file)
            .map_or_else(Self::default, |path| {
                Self::load_from_file(&path).unwrap_or_else(|e| {
                    eprintln!("Error loading config from {}: {e}", path.display());
                    process::exit(2);
                })
            })
    }

    #[must_use]
    pub fn rule_lint_level_in_conf(&self, rule_id: &str) -> Option<LintLevel> {
        let mut rule_lint_level_within_conf = None;
        for (set_name, level) in &self.lints.sets {
            if set_name.rules.contains_key(rule_id) {
                if let Some(rule_lint_level_within_lint_set) = &set_name.rules[rule_id] {
                    match rule_lint_level_within_conf {
                        None => {
                            rule_lint_level_within_conf = Some(*rule_lint_level_within_lint_set);
                        }
                        Some(existing_level) => {
                            // Choose the more severe level
                            if *rule_lint_level_within_lint_set > existing_level {
                                rule_lint_level_within_conf =
                                    Some(*rule_lint_level_within_lint_set);
                            }
                        }
                    }
                } else {
                    match rule_lint_level_within_conf {
                        None => rule_lint_level_within_conf = Some(*level),
                        Some(existing_level) => {
                            if level > &existing_level {
                                rule_lint_level_within_conf = Some(*level);
                            }
                        }
                    }
                }
            }
        }
        rule_lint_level_within_conf
    }

    /// Get the effective lint level for a specific rule
    /// Priority: individual rule config > any applicable set config > rule default
    #[must_use]
    pub fn get_lint_level(&self, rule_id: &str, default_level: LintLevel) -> LintLevel {
        // Check individual rule configuration first
        if let Some(level) = self.rule_lint_level_in_conf(rule_id) {
            return level;
        }

        // Fall back to rule's default
        default_level
    }
}

/// Search for .nu-lint.toml in current directory and parent directories
#[must_use]
pub fn find_config_file() -> Option<PathBuf> {
    let mut current_dir = current_dir().ok()?;

    loop {
        let config_path = current_dir.join(".nu-lint.toml");
        if config_path.exists() && config_path.is_file() {
            return Some(config_path);
        }

        // Try to go to parent directory
        if !current_dir.pop() {
            break;
        }
    }

    None
}
