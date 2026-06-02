use std::{fs, path::PathBuf};

use crate::{
    LintError, LintLevel,
    config::{Config, user_config_path},
};

pub const DISABLE_RULE_COMMAND: &str = "nu-lint.disableRule";

/// Execute the disable rule action by writing to the user-wide XDG config
/// file. Returns the path that was modified on success.
pub fn execute_disable_rule(rule_id: &str) -> Result<PathBuf, LintError> {
    let config_path = user_config_path().ok_or(LintError::NoConfigLocation)?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|source| LintError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let content = fs::read_to_string(&config_path).unwrap_or_default();

    let mut config = if content.is_empty() {
        Config::default()
    } else {
        Config::load_from_str(&content)?
    };

    if config.rules.get(rule_id) == Some(&LintLevel::Off) {
        return Ok(config_path);
    }

    config.rules.insert(rule_id.to_string(), LintLevel::Off);

    let new_content =
        toml::to_string_pretty(&config).map_err(|source| LintError::ConfigSerialize { source })?;

    fs::write(&config_path, &new_content).map_err(|source| LintError::Io {
        path: config_path.clone(),
        source,
    })?;

    tracing::info!("Disabled rule '{}' in {}", rule_id, config_path.display());

    Ok(config_path)
}
