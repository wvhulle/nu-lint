#![allow(clippy::missing_errors_doc, reason = "Necessary for testing.")]
use core::fmt::{self, Display};
use std::{
    collections::HashMap,
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    process,
};

use serde::{Deserialize, Serialize};

use crate::{LintError, lint_set::builtin_lint_sets, rules::RuleRegistry};

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
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LintConfig {
    /// Configure entire lint sets (e.g., "naming", "idioms", "pedantic")
    #[serde(default)]
    pub sets: HashMap<String, LintLevel>,

    /// Configure individual rules (overrides set settings)
    #[serde(default)]
    pub rules: HashMap<String, LintLevel>,
}

impl<'de> Deserialize<'de> for LintConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LintConfigHelper {
            #[serde(default)]
            sets: HashMap<String, LintLevel>,
            #[serde(default)]
            rules: HashMap<String, LintLevel>,
        }

        let helper = LintConfigHelper::deserialize(deserializer)?;

        Ok(Self {
            sets: helper.sets,
            rules: helper.rules,
        })
    }
}

impl Default for LintConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();

        let registry = RuleRegistry::with_default_rules();

        for rule in registry.all_rules() {
            rules.insert(rule.id.to_string(), rule.default_lint_level);
        }

        Self {
            sets: HashMap::new(),
            rules,
        }
    }
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
    pub fn load_from_str(toml_str: &str) -> Result<Self, LintError> {
        Ok(toml::from_str(toml_str)?)
    }
    /// Load configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the TOML content is
    /// invalid.
    pub fn load_from_file(path: &Path) -> Result<Self, LintError> {
        let content = fs::read_to_string(path)?;
        Self::load_from_str(&content)
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

        // Check if the rule belongs to any configured lint sets
        let builtin_sets = builtin_lint_sets();
        for (set_name, level) in &self.lints.sets {
            log::debug!("Lint set {set_name} is enabled with level {level:?} in config");

            // Look up the set in builtin sets
            if let Some(lint_set) = builtin_sets.get(set_name.as_str()) {
                // Check if this rule is in the set
                if lint_set.rules.contains_key(rule_id) {
                    log::debug!("Rule '{rule_id}' found in set '{set_name}' with level {level:?}");
                    match rule_lint_level_within_conf {
                        None => rule_lint_level_within_conf = Some(*level),
                        Some(existing_level) => {
                            if *level > existing_level {
                                rule_lint_level_within_conf = Some(*level);
                            }
                        }
                    }
                }
            }
        }

        // Individual rule configuration overrides set configuration
        if let Some(level) = self.lints.rules.get(rule_id) {
            log::debug!(
                "Rule '{rule_id}' has individual level '{level:?}' in config, overriding set levels"
            );
            rule_lint_level_within_conf = Some(*level);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::instrument;

    #[test]
    fn test_load_config_simple_str() {
        let toml_str = r#"
        [lints.rules]
        snake_case_variables = "deny"
    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(
            config.lints.rules.get("snake_case_variables"),
            Some(&LintLevel::Deny)
        );
    }

    #[test]
    fn test_load_config_simple_str_set() {
        let toml_str = r#"
        [lints.sets]
        naming = "deny"
    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        let found_set_level = config.lints.sets.iter().find(|(k, _)| **k == "naming");
        assert!(matches!(found_set_level, Some((_, LintLevel::Deny))));
    }

    #[test]
    fn test_load_config_load_from_set_deny() {
        let toml_str = r#"
        [lints.sets]
        naming = "deny"
    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        let found_set_level = config.rule_lint_level_in_conf("snake_case_variables");
        assert!(matches!(found_set_level, Some(LintLevel::Deny)));
    }

    #[test]
    fn test_load_config_load_from_set_allow() {
        instrument();
        let toml_str = r#"
        [lints.sets]
        naming = "allow"

    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        let found_set_level = config.rule_lint_level_in_conf("snake_case_variables");
        assert!(matches!(found_set_level, Some(LintLevel::Allow)));
    }

    #[test]
    fn test_load_config_load_from_set_deny_empty() {
        instrument();
        let toml_str = r"
    ";

        let config = Config::load_from_str(toml_str).unwrap();
        let found_set_level = config.rule_lint_level_in_conf("snake_case_variables");
        assert!(matches!(found_set_level, Some(LintLevel::Allow)));
    }

    #[test]
    fn test_load_config_load_from_set_deny_conflict() {
        instrument();
        let toml_str = r#"
        [lints.sets]
        naming = "deny"
        [lints.rules]
        snake_case_variables = "allow"
    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        let found_set_level = config.rule_lint_level_in_conf("snake_case_variables");
        assert_eq!(found_set_level, Some(LintLevel::Allow));
    }
}
