use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    LintError,
    config::{Config, ToggledLevel},
};

pub const DISABLE_RULE_COMMAND: &str = "nu-lint.disableRule";

/// Execute the disable rule action by writing to the config file.
/// Returns the path that was modified on success.
pub fn execute_disable_rule(
    workspace_root: Option<&Path>,
    rule_id: &str,
) -> Result<PathBuf, LintError> {
    let base_dir = workspace_root
        .map(Path::to_path_buf)
        .or_else(dirs::home_dir)
        .ok_or(LintError::NoConfigLocation)?;
    let config_path = base_dir.join(".nu-lint.toml");

    let content = fs::read_to_string(&config_path).unwrap_or_default();
    let mut config = if content.is_empty() {
        Config::default()
    } else {
        Config::load_from_str(&content)?
    };

    if config.rules.get(rule_id) == Some(&ToggledLevel(None)) {
        return Ok(config_path);
    }

    config.rules.insert(rule_id.to_string(), ToggledLevel(None));

    let new_content =
        toml::to_string_pretty(&config).map_err(|source| LintError::ConfigSerialize { source })?;

    fs::write(&config_path, new_content).map_err(|source| LintError::Io {
        path: config_path.clone(),
        source,
    })?;

    Ok(config_path)
}
