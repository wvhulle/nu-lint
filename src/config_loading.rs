use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use nu_lint_analysis::config::{Config, LintLevel};
use nu_lint_rules::{USED_RULES, groups::ALL_GROUPS};

use crate::LintError;

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

    if let Some(unknown_group_name) = config.groups.keys().find(|group_name| {
        !ALL_GROUPS
            .iter()
            .any(|group| group.name == group_name.as_str())
    }) {
        return Err(LintError::GroupDoesNotExist {
            non_existing_name: unknown_group_name.clone(),
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

fn toml_level(level: LintLevel) -> toml::Value {
    toml::Value::try_from(level).expect("lint level serializes to a toml value")
}

fn commented_groups_toml() -> String {
    ALL_GROUPS.iter().fold(
        String::from(
            "# Set the level of every rule in a group at once, for example to turn it \
             off\n[groups]\n",
        ),
        |mut groups_toml, group| {
            writeln!(
                groups_toml,
                "# {}\n# {} = {}",
                group.description,
                group.name,
                toml_level(LintLevel::Off)
            )
            .unwrap();
            groups_toml
        },
    )
}

fn commented_rules_toml() -> String {
    ALL_GROUPS.iter().fold(
        String::from(
            "# Set the level of a single rule, overriding its group. Shown are the default \
             levels.\n[rules]\n",
        ),
        |mut rules_toml, group| {
            writeln!(rules_toml, "\n# {}", group.name).unwrap();
            for rule in group.rules {
                writeln!(rules_toml, "# {} = {}", rule.id(), toml_level(rule.level())).unwrap();
            }
            rules_toml
        },
    )
}

#[must_use]
pub fn example_config_toml() -> String {
    format!(
        "{}\n{}\n{}",
        Config::documented_default_toml(),
        commented_groups_toml(),
        commented_rules_toml()
    )
}

#[must_use]
pub fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|config_dir| config_dir.join("nu-lint.toml"))
}

pub fn load_validated_config(explicit_path: Option<&Path>) -> Result<Config, LintError> {
    let config_path = explicit_path
        .map(Path::to_path_buf)
        .or_else(|| user_config_path().filter(|path| path.is_file()));
    let config = match config_path {
        Some(path) => read_config_file(&path)?,
        None => Config::default(),
    };
    validate_rule_levels(&config)?;
    Ok(config)
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

    #[test]
    fn test_validate_rejects_unknown_group() {
        let config = parse_config_toml("[groups]\nfiltering = \"off\"").unwrap();
        assert!(matches!(
            validate_rule_levels(&config),
            Err(LintError::GroupDoesNotExist { non_existing_name }) if non_existing_name == "filtering"
        ));
    }

    fn uncommented_level_lines(commented_toml: &str) -> String {
        commented_toml
            .lines()
            .map(|line| {
                line.strip_prefix("# ")
                    .filter(|uncommented| {
                        uncommented
                            .split_once(" = ")
                            .is_some_and(|(key, _)| !key.contains(' '))
                    })
                    .unwrap_or(line)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn example_config_parses_to_default_config() {
        assert_eq!(
            parse_config_toml(&example_config_toml()).unwrap(),
            Config::default()
        );
    }

    #[test]
    fn uncommented_example_rules_keep_default_levels() {
        let uncommented_config =
            parse_config_toml(&uncommented_level_lines(&commented_rules_toml())).unwrap();
        validate_rule_levels(&uncommented_config).unwrap();
        assert_eq!(uncommented_config.rules.len(), USED_RULES.len());
        assert!(
            USED_RULES.iter().all(|rule| {
                uncommented_config.get_lint_level(*rule, ALL_GROUPS) == rule.level()
            })
        );
    }

    #[test]
    fn each_uncommented_example_group_is_valid_alone() {
        let invalid_group_lines: Vec<_> = uncommented_level_lines(&commented_groups_toml())
            .lines()
            .filter(|line| !line.starts_with('#') && line.contains(" = "))
            .filter(|group_line| {
                parse_config_toml(&format!("[groups]\n{group_line}"))
                    .and_then(|group_config| validate_rule_levels(&group_config))
                    .is_err()
            })
            .map(ToString::to_string)
            .collect();
        assert!(
            invalid_group_lines.is_empty(),
            "example group lines rejected on their own: {invalid_group_lines:?}"
        );
    }

    #[test]
    fn test_validate_accepts_known_group() {
        let config = parse_config_toml("[groups]\npipe-instead-of-loop = \"off\"").unwrap();
        assert!(validate_rule_levels(&config).is_ok());
    }
}
