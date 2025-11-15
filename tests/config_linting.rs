use std::{
    env::{current_dir, set_current_dir},
    fs,
    path::PathBuf,
    sync::Mutex,
};

use nu_lint::{
    LintEngine,
    cli::{collect_files_to_lint, lint_files},
    config::{Config, LintLevel},
};
use tempfile::TempDir;

pub static CHDIR_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_custom_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("custom.toml");
    let nu_file_path = temp_dir.path().join("test.nu");

    fs::write(
        &config_path,
        "[lints]\n\n[lints.rules]\nsnake_case_variables = \"deny\"\n",
    )
    .unwrap();
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let config = Config::load(Some(&config_path));
    assert_eq!(
        config.lints.rules.get("snake_case_variables"),
        Some(&LintLevel::Deny)
    );

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
        "[lints.rules]\nsnake_case_variables = \"allow\"\n",
    )
    .unwrap();
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let original_dir = current_dir().unwrap();

    set_current_dir(temp_dir.path()).unwrap();

    let config = Config::load(None);
    let engine = LintEngine::new(config);
    let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
    let (violations, _) = lint_files(&engine, &files, false);

    set_current_dir(original_dir).unwrap();

    // Should have no violations because snake_case_variables is allowed
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
        "[lints.rules]\nsnake_case_variables = \"allow\"\n",
    )
    .unwrap();
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let original_dir = current_dir().unwrap();

    set_current_dir(&subdir).unwrap();

    let config = Config::load(None);
    let engine = LintEngine::new(config);
    let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
    let (violations, _) = lint_files(&engine, &files, false);

    set_current_dir(original_dir).unwrap();

    // Should have no violations because snake_case_variables is allowed
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
        "[lints.rules]\nsnake_case_variables = \"allow\"\n",
    )
    .unwrap();
    fs::write(&explicit_config, "[lints.rules]\n").unwrap();
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let original_dir = current_dir().unwrap();

    set_current_dir(temp_dir.path()).unwrap();

    // Explicit config should override auto-discovery
    let config = Config::load(Some(&explicit_config));
    let engine = LintEngine::new(config);
    let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
    let (violations, _) = lint_files(&engine, &files, false);

    set_current_dir(original_dir).unwrap();

    // Should have violations because explicit config doesn't allow the rule
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "snake_case_variables")
    );
}
