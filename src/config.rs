use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::IntoDeserializer};

use crate::{
    LintError,
    rule::Rule,
    rules::{USED_RULES, groups::ALL_GROUPS},
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    Hint,
    #[default]
    Warning,
    Error,
}

/// Wrapper for `Option<LintLevel>` that serializes `None` as `"off"`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ToggledLevel(pub Option<LintLevel>);

impl Serialize for ToggledLevel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Some(level) => level.serialize(serializer),
            None => serializer.serialize_str("off"),
        }
    }
}

impl<'de> Deserialize<'de> for ToggledLevel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "off" => Ok(Self(None)),
            _ => LintLevel::deserialize(s.into_deserializer()).map(|l| Self(Some(l))),
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
    pub rules: HashMap<String, ToggledLevel>,
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

        if let Some(ToggledLevel(level)) = self.rules.get(rule_id) {
            log::debug!(
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

            log::debug!("Rule '{rule_id}' found in set '{set_name}' with level {level:?}");
            return Some(*level);
        }

        rule.level()
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

    #[test]
    fn test_load_config_simple_str() {
        let toml_str = r#"
        [rules]
        snake_case_variables = "error"
        other_rule = "off"
    "#;

        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(
            config.rules["snake_case_variables"],
            ToggledLevel(Some(LintLevel::Error))
        );

        assert_eq!(config.rules["other_rule"], ToggledLevel(None));
    }

    #[test]
    fn test_validate_passes_with_default_config() {
        let result = Config::default().validate();
        assert!(result.is_ok());
    }
}
