mod common;

use std::fs;
use std::path::PathBuf;

use nu_lint::cli::{collect_files_to_lint, lint_files};
use nu_lint::config::{RuleSeverity, load_config};
use nu_lint::engine::LintEngine;
use tempfile::TempDir;

use common::CHDIR_MUTEX;

#[test]
fn test_custom_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("custom.toml");
    let nu_file_path = temp_dir.path().join("test.nu");

    fs::write(&config_path, "[general]\nmin_severity = \"info\"\n").unwrap();
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let config = load_config(Some(&config_path));
    assert_eq!(config.general.min_severity, RuleSeverity::Info);

    let engine = LintEngine::new(config);
    let files = collect_files_to_lint(&[nu_file_path]);
    let (violations, _) = lint_files(&engine, &files, false);

    assert!(!violations.is_empty());
}

#[test]
fn test_auto_discover_config_file() {
    let _guard = CHDIR_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".nu-lint.toml");
    let nu_file_path = temp_dir.path().join("test.nu");

    fs::write(
        &config_path,
        "[general]\nmin_severity = \"info\"\n\n[rules]\nsnake_case_variables = \"off\"\n",
    )
    .unwrap();
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let original_dir = std::env::current_dir().unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = load_config(None);
        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files, false);

        violations
    }));

    std::env::set_current_dir(original_dir).unwrap();

    let violations = result.unwrap();

    // Should have no violations because snake_case_variables is off
    assert!(
        violations
            .iter()
            .all(|v| v.rule_id != "snake_case_variables")
    );
}

#[test]
fn test_auto_discover_config_in_parent_dir() {
    let _guard = CHDIR_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".nu-lint.toml");
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let nu_file_path = subdir.join("test.nu");

    fs::write(
        &config_path,
        "[general]\nmin_severity = \"info\"\n\n[rules]\nsnake_case_variables = \"off\"\n",
    )
    .unwrap();
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let original_dir = std::env::current_dir().unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(&subdir).unwrap();

        let config = load_config(None);
        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files, false);

        violations
    }));

    std::env::set_current_dir(original_dir).unwrap();

    let violations = result.unwrap();

    // Should have no violations because snake_case_variables is off
    assert!(
        violations
            .iter()
            .all(|v| v.rule_id != "snake_case_variables")
    );
}

#[test]
fn test_explicit_config_overrides_auto_discovery() {
    let _guard = CHDIR_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let auto_config = temp_dir.path().join(".nu-lint.toml");
    let explicit_config = temp_dir.path().join("other.toml");
    let nu_file_path = temp_dir.path().join("test.nu");

    fs::write(
        &auto_config,
        "[general]\nmin_severity = \"info\"\n\n[rules]\nsnake_case_variables = \"off\"\n",
    )
    .unwrap();
    fs::write(
        &explicit_config,
        "[general]\nmin_severity = \"info\"\n\n[rules]\n",
    )
    .unwrap();
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let original_dir = std::env::current_dir().unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Explicit config should override auto-discovery
        let config = load_config(Some(&explicit_config));
        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files, false);

        violations
    }));

    std::env::set_current_dir(original_dir).unwrap();

    let violations = result.unwrap();

    // Should have violations because explicit config doesn't disable the rule
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "snake_case_variables")
    );
}
