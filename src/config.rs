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
    /// When true, rules recommend `get --optional` instead of `$list.0?` for
    /// safe access. Default is false (prefer `?` syntax).
    pub explicit_optional_access: bool,
    /// Maximum number of top-level statements allowed in a function body before
    /// `max_function_body_length` fires. Default: 40.
    pub max_function_body_statements: usize,
    /// Maximum number of pipeline elements (pipe-separated steps) a
    /// single-call function body may have before `single_call_command` stops
    /// suggesting it can be inlined. Default: 2 (at most one `|`).
    pub max_inlinable_pipeline_elements: usize,
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
