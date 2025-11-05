use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process,
};

use serde::{Deserialize, Serialize};

use crate::violation::Severity;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GeneralConfig {
    #[serde(default = "GeneralConfig::default_min_severity")]
    pub min_severity: RuleSeverity,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            min_severity: Self::default_min_severity(),
        }
    }
}

impl GeneralConfig {
    const fn default_min_severity() -> RuleSeverity {
        RuleSeverity::Warning // Show warnings and errors by default
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, PartialOrd, Ord, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Off,
    Info,
    Warning,
    #[default]
    Error,
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

impl From<Severity> for RuleSeverity {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Info => Self::Info,
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
    /// Load configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the TOML content is
    /// invalid.
    pub(crate) fn load_from_file(path: &Path) -> Result<Self, crate::LintError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
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

    pub(crate) fn rule_severity(&self, rule_id: &str) -> Option<Severity> {
        self.rules.get(rule_id).copied().and_then(Into::into)
    }
}

/// Search for .nu-lint.toml in current directory and parent directories
#[must_use]
pub fn find_config_file() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;

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
