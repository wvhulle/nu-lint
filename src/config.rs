use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process,
};

use serde::{Deserialize, Serialize};

use crate::lint::Severity;

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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct GeneralConfig {
    pub max_severity: RuleSeverity,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct ExcludeConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
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

/// Load configuration from file or use defaults
#[must_use]
pub fn load_config(config_path: Option<&PathBuf>) -> Config {
    let path = config_path.cloned().or_else(find_config_file);

    if let Some(path) = path {
        Config::load_from_file(&path).unwrap_or_else(|e| {
            eprintln!("Error loading config from {}: {e}", path.display());
            process::exit(2);
        })
    } else {
        Config::default()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::test_utils::CHDIR_MUTEX;

    fn with_temp_dir<F>(f: F)
    where
        F: FnOnce(&TempDir),
    {
        let _guard = CHDIR_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Use catch_unwind to ensure directory is restored even if the test panics
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            std::env::set_current_dir(temp_dir.path()).unwrap();
            f(&temp_dir);
        }));

        // Always restore directory, even if test panics
        std::env::set_current_dir(original_dir).unwrap();

        // Re-panic if the test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_find_config_file_in_current_dir() {
        with_temp_dir(|temp_dir| {
            let config_path = temp_dir.path().join(".nu-lint.toml");
            fs::write(&config_path, "[rules]\n").unwrap();

            let found = find_config_file();
            assert!(found.is_some());
            assert_eq!(found.unwrap(), config_path);
        });
    }

    #[test]
    fn test_find_config_file_in_parent_dir() {
        let _guard = CHDIR_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".nu-lint.toml");
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(&config_path, "[rules]\n").unwrap();

        let original_dir = std::env::current_dir().unwrap();

        // Use a closure with defer-like behavior to ensure directory is restored
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            std::env::set_current_dir(&subdir).unwrap();
            find_config_file()
        }));

        // Always restore directory, even if test panics
        std::env::set_current_dir(original_dir).unwrap();

        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap(), config_path);
    }

    #[test]
    fn test_find_config_file_not_found() {
        with_temp_dir(|_temp_dir| {
            let found = find_config_file();
            assert!(found.is_none());
        });
    }

    #[test]
    fn test_load_config_with_explicit_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\nmax_severity = \"error\"\n").unwrap();

        let config = load_config(Some(&config_path));
        assert_eq!(config.general.max_severity, RuleSeverity::Error);
    }

    #[test]
    fn test_load_config_auto_discover() {
        let _guard = CHDIR_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".nu-lint.toml");
        fs::write(&config_path, "[general]\nmax_severity = \"warning\"\n").unwrap();

        let original_dir = std::env::current_dir().unwrap();

        // Use a closure with defer-like behavior to ensure directory is restored
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            std::env::set_current_dir(temp_dir.path()).unwrap();
            load_config(None)
        }));

        // Always restore directory, even if test panics
        std::env::set_current_dir(original_dir).unwrap();

        let config = result.unwrap();
        assert_eq!(config.general.max_severity, RuleSeverity::Warning);
    }

    #[test]
    fn test_load_config_default() {
        with_temp_dir(|_temp_dir| {
            let config = load_config(None);
            assert_eq!(config, Config::default());
        });
    }
}
