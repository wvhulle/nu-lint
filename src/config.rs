use core::fmt::{self, Display};
use std::{
    collections::HashMap,
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    process,
};

use serde::{Deserialize, Serialize};

use crate::{
    LintError,
    rules::sets::{BUILTIN_LINT_SETS, RULE_LEVEL_OVERRIDES},
};

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

#[derive(Deserialize)]
#[serde(untagged)]
enum ConfigField {
    Lints(LintConfig),
    Sequential(bool),
    Level(LintLevel),
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub lints: LintConfig,

    /// Process files sequentially instead of in parallel (useful for debugging)
    #[serde(default)]
    pub sequential: bool,
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = HashMap::<String, ConfigField>::deserialize(deserializer)?;

        let mut lints = None;
        let mut sequential = None;
        let mut bare_items = HashMap::new();

        for (key, value) in map {
            match (key.as_str(), value) {
                ("lints", ConfigField::Lints(l)) => lints = Some(l),
                ("sequential", ConfigField::Sequential(s)) => sequential = Some(s),
                (_, ConfigField::Level(level)) => {
                    bare_items.insert(key, level);
                }
                _ => {}
            }
        }

        let mut lints = lints.unwrap_or_default();
        merge_bare_items_into_lints(&mut lints, bare_items);

        Ok(Self {
            lints,
            sequential: sequential.unwrap_or(false),
        })
    }
}

fn merge_bare_items_into_lints(lints: &mut LintConfig, bare_items: HashMap<String, LintLevel>) {
    for (name, level) in bare_items {
        let is_set = BUILTIN_LINT_SETS.iter().any(|set| set.name == name);

        if is_set {
            lints.sets.insert(name, level);
        } else {
            lints.rules.insert(name, level);
        }
    }
}

/// Lint configuration with support for set-level and individual rule
/// configuration
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct ExcludeConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl Config {
    /// Load configuration from a TOML string.
    ///
    /// # Errors
    ///
    /// Errors when TOML string is not a valid TOML string.
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

    /// Get the effective lint level for a specific rule
    /// Priority (high to low):
    /// 1. Individual rule level in config
    /// 2. Lint set level in config (highest level if rule appears in multiple
    ///    sets)
    /// 3. Default level from default rule map
    #[must_use]
    pub fn get_lint_level(&self, rule_id: &'static str) -> LintLevel {
        if let Some(level) = self.lints.rules.get(rule_id) {
            log::debug!(
                "Rule '{rule_id}' has individual level '{level:?}' in config, overriding set \
                 levels"
            );
            return *level;
        }

        let mut max_level: Option<LintLevel> = None;

        for (set_name, level) in &self.lints.sets {
            let Some(lint_set) = BUILTIN_LINT_SETS
                .iter()
                .find(|set| set.name == set_name.as_str())
            else {
                continue;
            };

            if !lint_set.rules.iter().any(|rule| rule.id == rule_id) {
                continue;
            }

            log::debug!("Rule '{rule_id}' found in set '{set_name}' with level {level:?}");
            max_level = Some(max_level.map_or(*level, |existing| existing.max(*level)));
        }

        max_level.unwrap_or_else(|| {
            RULE_LEVEL_OVERRIDES
                .rules
                .iter()
                .find(|(rule, _)| rule.id == rule_id)
                .map(|(_, level)| level)
                .copied()
                .unwrap_or(LintLevel::Warn)
        })
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
        let found_set_level = config.get_lint_level("snake_case_variables");
        assert_eq!(found_set_level, LintLevel::Deny);
    }

    #[test]
    fn test_load_config_load_from_set_allow() {
        instrument();
        let toml_str = r#"
        [lints.sets]
        naming = "allow"

    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        let found_set_level = config.get_lint_level("snake_case_variables");
        assert_eq!(found_set_level, LintLevel::Allow);
    }

    #[test]
    fn test_load_config_load_from_set_deny_empty() {
        instrument();
        let toml_str = r"
    ";

        let config = Config::load_from_str(toml_str).unwrap();
        let found_set_level = config.get_lint_level("snake_case_variables");
        assert_eq!(found_set_level, LintLevel::Warn);
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
        let found_set_level = config.get_lint_level("snake_case_variables");
        assert_eq!(found_set_level, LintLevel::Allow);
    }

    #[test]
    fn test_bare_rule_format() {
        let toml_str = r#"
        snake_case_variables = "deny"
        systemd_journal_prefix = "warn"
    "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(
            config.lints.rules.get("snake_case_variables"),
            Some(&LintLevel::Deny)
        );
        assert_eq!(
            config.lints.rules.get("systemd_journal_prefix"),
            Some(&LintLevel::Warn)
        );
    }

    #[test]
    fn test_bare_set_format() {
        let toml_str = r#"
        naming = "deny"
        performance = "warn"
    "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(config.lints.sets.get("naming"), Some(&LintLevel::Deny));
        assert_eq!(config.lints.sets.get("performance"), Some(&LintLevel::Warn));
    }

    #[test]
    fn test_mixed_bare_and_structured() {
        let toml_str = r#"
        naming = "deny"
        systemd_journal_prefix = "warn"
        
        [lints.rules]
        snake_case_variables = "allow"
    "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(config.lints.sets.get("naming"), Some(&LintLevel::Deny));
        // Bare rules get added to rules
        assert_eq!(
            config.lints.rules.get("systemd_journal_prefix"),
            Some(&LintLevel::Warn)
        );
        // Structured format values are merged with bare format
        assert_eq!(
            config.lints.rules.get("snake_case_variables"),
            Some(&LintLevel::Allow)
        );
    }

    #[test]
    fn test_bare_format_resolves_level() {
        let toml_str = r#"
        naming = "deny"
    "#;
        let config = Config::load_from_str(toml_str).unwrap();
        // snake_case_variables is in the naming set
        assert_eq!(
            config.get_lint_level("snake_case_variables"),
            LintLevel::Deny
        );
    }
}
