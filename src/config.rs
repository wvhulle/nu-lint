use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use miette::Severity;
use serde::{Deserialize, Serialize};

use crate::{
    LintError,
    rule::Rule,
    rules::{USED_RULES, groups::ALL_GROUPS},
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    Off,
    Hint,
    #[default]
    Warning,
    Error,
}

impl TryFrom<LintLevel> for Severity {
    type Error = ();
    fn try_from(value: LintLevel) -> Result<Self, ()> {
        match value {
            LintLevel::Off => Err(()),
            LintLevel::Hint => Ok(Self::Advice),
            LintLevel::Warning => Ok(Self::Warning),
            LintLevel::Error => Ok(Self::Error),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PipelinePlacement {
    #[default]
    Start,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub groups: HashMap<String, LintLevel>,
    pub rules: HashMap<String, LintLevel>,
    pub sequential: bool,
    pub pipeline_placement: PipelinePlacement,
    pub max_pipeline_length: usize,
    pub skip_external_parse_errors: bool,
    /// When true, rules recommend `get --optional` instead of `$list.0?` for
    /// safe access. Default is false (prefer `?` syntax).
    pub explicit_optional_access: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            groups: HashMap::new(),
            rules: HashMap::new(),
            sequential: false,
            pipeline_placement: PipelinePlacement::default(),
            max_pipeline_length: 80,
            skip_external_parse_errors: true,
            explicit_optional_access: false,
        }
    }
}

impl Config {
    /// Load configuration from a TOML string.
    ///
    /// # Errors
    ///
    /// Errors when TOML string is not a valid TOML string.
    pub(crate) fn load_from_str(toml_str: &str) -> Result<Self, LintError> {
        toml::from_str(toml_str).map_err(|source| LintError::Config { source })
    }
    /// Load configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the TOML content is
    /// invalid.
    pub(crate) fn load_from_file(path: &Path) -> Result<Self, LintError> {
        log::debug!("Loading configuration file at {}", path.display());
        let content = fs::read_to_string(path).map_err(|source| LintError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::load_from_str(&content)
    }

    /// Validate that no conflicting rules are both enabled
    ///
    /// # Errors
    ///
    /// Returns an error if two conflicting rules are both enabled.
    pub fn validate(&self) -> Result<(), LintError> {
        log::debug!("Validating loaded configuration.");

        for rule_id_in_config_file in self.rules.keys() {
            if USED_RULES
                .iter()
                .find(|rule| rule.id() == rule_id_in_config_file)
                .is_none()
            {
                return Err(LintError::RuleDoesNotExist {
                    non_existing_id: rule_id_in_config_file.clone(),
                });
            }
        }

        for rule in USED_RULES {
            if self.get_lint_level(*rule) == LintLevel::Off {
                continue;
            }

            for conflicting_rule in rule.conflicts_with() {
                if self.get_lint_level(*conflicting_rule) > LintLevel::Off {
                    return Err(LintError::RuleConflict {
                        rule_a: rule.id(),
                        rule_b: conflicting_rule.id(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Get the effective lint level for a specific rule
    #[must_use]
    pub fn get_lint_level(&self, rule: &dyn Rule) -> LintLevel {
        let rule_id = rule.id();

        if let Some(level) = self.rules.get(rule_id) {
            log::trace!(
                "Rule '{rule_id}' has individual level '{level:?}' in config, overriding set \
                 levels"
            );
            return *level;
        }

        for (set_name, level) in &self.groups {
            let Some(lint_set) = ALL_GROUPS.iter().find(|set| set.name == set_name.as_str()) else {
                continue;
            };

            if !lint_set.rules.iter().any(|r| r.id() == rule_id) {
                continue;
            }

            log::trace!("Rule '{rule_id}' found in set '{set_name}' with level {level:?}");
            return *level;
        }

        rule.level()
    }
}

/// Search for `.nu-lint.toml` in the given directory, falling back to home
/// directory
#[must_use]
pub fn find_config_file_from(start_dir: &Path) -> Option<PathBuf> {
    // Check active directory first
    let config_path = start_dir.join(".nu-lint.toml");
    if config_path.exists() && config_path.is_file() {
        return Some(config_path);
    }

    // Fall back to home directory
    let home_config = dirs::home_dir()?.join(".nu-lint.toml");
    if home_config.exists() && home_config.is_file() {
        return Some(home_config);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_simple_str() {
        let toml_str = r#"
        [rules]
        snake_case_variables = "error"
        other_rule = "off"
    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(config.rules["snake_case_variables"], LintLevel::Error);

        assert_eq!(config.rules["other_rule"], LintLevel::Off);
    }

    #[test]
    fn test_validate_passes_with_default_config() {
        let result = Config::default().validate();
        assert!(result.is_ok());
    }
}
