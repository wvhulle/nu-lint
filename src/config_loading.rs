use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    LintError,
    config::{Config, LintLevel},
    rules::{USED_RULES, groups::ALL_GROUPS},
};

pub fn parse_config_toml(toml_str: &str) -> Result<Config, LintError> {
    toml::from_str(toml_str).map_err(|source| LintError::Config { source })
}

pub fn read_config_file(path: &Path) -> Result<Config, LintError> {
    log::debug!("Loading configuration file at {}", path.display());
    let content = fs::read_to_string(path).map_err(|source| LintError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_config_toml(&content)
}

pub fn validate_rule_levels(config: &Config) -> Result<(), LintError> {
    log::debug!("Validating loaded configuration.");

    if let Some(unknown_rule_id) = config
        .rules
        .keys()
        .find(|rule_id| !USED_RULES.iter().any(|rule| rule.id() == rule_id.as_str()))
    {
        return Err(LintError::RuleDoesNotExist {
            non_existing_id: unknown_rule_id.clone(),
        });
    }

    for rule in USED_RULES {
        if config.get_lint_level(*rule, ALL_GROUPS) == LintLevel::Off {
            continue;
        }

        for conflicting_rule in rule.conflicts_with() {
            if config.get_lint_level(*conflicting_rule, ALL_GROUPS) > LintLevel::Off {
                return Err(LintError::RuleConflict {
                    rule_a: rule.id(),
                    rule_b: conflicting_rule.id(),
                });
            }
        }
    }
    Ok(())
}

#[must_use]
pub fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|config_dir| config_dir.join("nu-lint.toml"))
}

#[must_use]
pub fn load_user_config() -> Config {
    user_config_path()
        .filter(|path| path.is_file())
        .and_then(|path| read_config_file(&path).ok())
        .unwrap_or_default()
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

        let config = parse_config_toml(toml_str).unwrap();
        assert_eq!(config.rules["snake_case_variables"], LintLevel::Error);

        assert_eq!(config.rules["other_rule"], LintLevel::Off);
    }

    #[test]
    fn test_validate_passes_with_default_config() {
        let result = validate_rule_levels(&Config::default());
        assert!(result.is_ok());
    }
}
