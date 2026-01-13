use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    LintError,
    rule::Rule,
    rules::{self, USED_RULES, groups::ALL_GROUPS},
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    Hint,
    #[default]
    Warning,
    Error,
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
    pub ignored: HashSet<String>,
    pub additional: HashSet<String>,
    pub sequential: bool,
    pub pipeline_placement: PipelinePlacement,
    pub max_pipeline_length: usize,
    pub skip_external_parse_errors: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            groups: HashMap::new(),
            rules: HashMap::new(),
            ignored: HashSet::from([
                rules::always_annotate_ext_hat::RULE.id().into(),
                rules::upstream::nu_parse_error::RULE.id().into(),
                rules::error_make::add_url::RULE.id().into(),
                rules::error_make::add_label::RULE.id().into(),
            ]),
            additional: HashSet::new(),
            sequential: false,
            pipeline_placement: PipelinePlacement::default(),
            max_pipeline_length: 80,
            skip_external_parse_errors: true,
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
        for rule in USED_RULES {
            if self.get_lint_level(*rule).is_none() {
                continue;
            }

            for conflicting_rule in rule.conflicts_with() {
                if self.get_lint_level(*conflicting_rule).is_some() {
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
    pub fn get_lint_level(&self, rule: &dyn Rule) -> Option<LintLevel> {
        let rule_id = rule.id();

        if self.ignored.contains(rule_id) {
            return None;
        }

        if let Some(level) = self.rules.get(rule_id) {
            log::debug!(
                "Rule '{rule_id}' has individual level '{level:?}' in config, overriding set \
                 levels"
            );
            return Some(*level);
        }

        for (set_name, level) in &self.groups {
            let Some(lint_set) = ALL_GROUPS.iter().find(|set| set.name == set_name.as_str()) else {
                continue;
            };

            if !lint_set.rules.iter().any(|r| r.id() == rule_id) {
                continue;
            }

            log::debug!("Rule '{rule_id}' found in set '{set_name}' with level {level:?}");
            return Some(*level);
        }

        Some(rule.level())
    }
}

/// Search for .nu-lint.toml starting from the given directory and walking up to
/// parent directories
#[must_use]
pub fn find_config_file_from(start_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = start_dir.to_path_buf();

    loop {
        let config_path = current_dir.join(".nu-lint.toml");
        if config_path.exists() && config_path.is_file() {
            return Some(config_path);
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::USED_RULES;

    #[test]
    fn test_load_config_simple_str() {
        let toml_str = r#"
        [rules]
        snake_case_variables = "error"
    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(
            config.rules.get("snake_case_variables"),
            Some(&LintLevel::Error)
        );
    }

    #[test]
    fn test_load_config_simple_str_set() {
        let toml_str = r#"
        ignored = [ "snake_case_variables" ]
        [groups]
        naming = "error"
    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        let found_set_level = config.groups.iter().find(|(k, _)| **k == "naming");
        assert!(matches!(found_set_level, Some((_, LintLevel::Error))));
        let ignored_rule = USED_RULES
            .iter()
            .find(|r| r.id() == "snake_case_variables")
            .unwrap();
        assert_eq!(config.get_lint_level(*ignored_rule), None);
    }

    #[test]
    fn test_validate_passes_with_default_config() {
        let result = Config::default().validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_passes_when_one_conflicting_rule_ignored() {
        // Enable add_hat but ignore remove_hat
        let mut ignored = Config::default().ignored;
        ignored.remove("add_hat_external_commands"); // enable add_hat
        ignored.insert("remove_hat_not_builtin".into()); // ignore remove_hat

        let config = Config {
            ignored,
            ..Config::default()
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_conflict_from_toml() {
        let toml_str = r#"
            ignored = []
        "#;

        let config = Config::load_from_str(toml_str).unwrap();
        let result = config.validate();

        assert!(result.is_err());
        assert!(matches!(result, Err(LintError::RuleConflict { .. })));
    }
}
