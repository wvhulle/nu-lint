use std::{collections::HashMap, sync::LazyLock};

use miette::Severity;
use serde::{Deserialize, Serialize};

use crate::rule::{Group, Rule};

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
    pub explicit_optional_access: bool,
    pub max_function_body_statements: usize,
    pub max_inlinable_pipeline_elements: usize,
}

fn documented_option(
    option_name: &str,
    description: &str,
    default_value: impl Serialize,
) -> String {
    let default_value =
        toml::Value::try_from(default_value).expect("config option serializes to a toml value");
    format!("# {description}\n{option_name} = {default_value}\n")
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
            max_function_body_statements: 40,
            max_inlinable_pipeline_elements: 2,
        }
    }
}

impl Config {
    /// Return a `&'static` reference to the default configuration.
    ///
    /// Useful when a `&'static Config` is needed without searching
    /// the file system (e.g. in tests).
    #[must_use]
    pub fn default_static() -> &'static Self {
        static DEFAULT: LazyLock<Config> = LazyLock::new(Config::default);
        &DEFAULT
    }

    #[must_use]
    pub fn documented_default_toml() -> String {
        let Self {
            groups: _,
            rules: _,
            sequential,
            pipeline_placement,
            max_pipeline_length,
            skip_external_parse_errors,
            explicit_optional_access,
            max_function_body_statements,
            max_inlinable_pipeline_elements,
        } = Self::default();
        [
            documented_option(
                "max_pipeline_length",
                "Maximum length of a single-line pipeline before reflow_wide_pipelines splits it \
                 over multiple lines",
                max_pipeline_length,
            ),
            documented_option(
                "pipeline_placement",
                "Where reflow_wide_pipelines puts the `|` when splitting a pipeline: \"start\" or \
                 \"end\" of a line",
                pipeline_placement,
            ),
            documented_option(
                "explicit_optional_access",
                "Recommend `get --optional` instead of `$list.0?` for safe access",
                explicit_optional_access,
            ),
            documented_option(
                "max_function_body_statements",
                "Maximum top-level statements in a function body before max_function_body_length \
                 fires",
                max_function_body_statements,
            ),
            documented_option(
                "max_inlinable_pipeline_elements",
                "Maximum pipe-separated steps a single-call function body may have before \
                 single_call_command stops suggesting it can be inlined",
                max_inlinable_pipeline_elements,
            ),
            documented_option(
                "skip_external_parse_errors",
                "Ignore parse errors in files that are imported but not linted themselves",
                skip_external_parse_errors,
            ),
            documented_option(
                "sequential",
                "Lint files one after another instead of in parallel",
                sequential,
            ),
        ]
        .join("\n")
    }

    #[must_use]
    pub fn get_lint_level(&self, rule: &dyn Rule, groups: &[Group]) -> LintLevel {
        let rule_id = rule.id();

        if let Some(level) = self.rules.get(rule_id) {
            log::trace!(
                "Rule '{rule_id}' has individual level '{level:?}' in config, overriding set \
                 levels"
            );
            return *level;
        }

        for (set_name, level) in &self.groups {
            let Some(lint_set) = groups.iter().find(|set| set.name == set_name.as_str()) else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_default_toml_parses_to_default_config() {
        let parsed_config: Config = toml::from_str(&Config::documented_default_toml()).unwrap();
        assert_eq!(parsed_config, Config::default());
    }

    #[test]
    fn documented_default_toml_sets_every_serialized_option() {
        let documented_options: toml::Table =
            toml::from_str(&Config::documented_default_toml()).unwrap();
        let mut serialized_options = toml::Table::try_from(Config::default()).unwrap();
        serialized_options
            .retain(|option_name, _| option_name != "groups" && option_name != "rules");
        assert_eq!(documented_options, serialized_options);
    }
}
