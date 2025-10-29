mod common;

use std::fs;

use common::{CHDIR_MUTEX, with_temp_dir};
use nu_lint::config::{Config, RuleSeverity, find_config_file, load_config};
use tempfile::TempDir;

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

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(&subdir).unwrap();
        find_config_file()
    }));

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
    fs::write(&config_path, "[general]\nmin_severity = \"error\"\n").unwrap();

    let config = load_config(Some(&config_path));
    assert_eq!(config.general.min_severity, RuleSeverity::Error);
}

#[test]
fn test_load_config_auto_discover() {
    let _guard = CHDIR_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".nu-lint.toml");
    fs::write(&config_path, "[general]\nmin_severity = \"warning\"\n").unwrap();

    let original_dir = std::env::current_dir().unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(temp_dir.path()).unwrap();
        load_config(None)
    }));

    std::env::set_current_dir(original_dir).unwrap();

    let config = result.unwrap();
    assert_eq!(config.general.min_severity, RuleSeverity::Warning);
}

#[test]
fn test_load_config_default() {
    with_temp_dir(|_temp_dir| {
        let config = load_config(None);
        assert_eq!(config, Config::default());
    });
}

#[test]
fn test_default_min_severity_is_warning() {
    let config = Config::default();
    assert_eq!(config.general.min_severity, RuleSeverity::Warning);
}
