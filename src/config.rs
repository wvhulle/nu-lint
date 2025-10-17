use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::lint::Severity;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub rules: HashMap<String, RuleSeverity>,

    #[serde(default)]
    pub style: StyleConfig,

    #[serde(default)]
    pub exclude: ExcludeConfig,

    #[serde(default)]
    pub fix: FixConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GeneralConfig {
    pub max_severity: RuleSeverity,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    #[default]
    Error,
    Warning,
    Info,
    Off,
}

impl From<RuleSeverity> for Option<Severity> {
    fn from(rule_sev: RuleSeverity) -> Self {
        match rule_sev {
            RuleSeverity::Error => Some(Severity::Error),
            RuleSeverity::Warning => Some(Severity::Warning),
            RuleSeverity::Info => Some(Severity::Info),
            RuleSeverity::Off => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ExcludeConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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
    pub fn load_from_file(path: &Path) -> Result<Self, crate::LintError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn rule_severity(&self, rule_id: &str) -> Option<Severity> {
        self.rules.get(rule_id).copied().and_then(Into::into)
    }
}
