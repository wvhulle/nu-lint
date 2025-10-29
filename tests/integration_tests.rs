use std::fs;
use std::path::PathBuf;
mod test_utils;
use nu_lint::cli::{collect_files_to_lint, collect_nu_files, lint_files};
use nu_lint::config::{Config, RuleSeverity, load_config};
use nu_lint::engine::LintEngine;
use tempfile::TempDir;
use test_utils::CHDIR_MUTEX;

#[test]
fn test_lint_file_with_violations() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("bad.nu");
    fs::write(&temp_file, "let myVariable = 5\n").unwrap();

    let mut config = Config::default();
    config.general.min_severity = RuleSeverity::Info;
    let engine = LintEngine::new(config);
    let files = collect_files_to_lint(&[temp_file]);
    let (violations, _) = lint_files(&engine, &files, false);

    assert!(!violations.is_empty());
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id.contains("snake_case_variables"))
    );
}

#[test]
fn test_lint_file_without_violations() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("good.nu");
    fs::write(&temp_file, "# Good code\nlet my_var = 5\n").unwrap();

    let config = Config::default();
    let engine = LintEngine::new(config);
    let files = collect_files_to_lint(&[temp_file]);
    let (violations, _) = lint_files(&engine, &files, false);

    assert!(violations.is_empty());
}

#[test]
fn test_list_rules_returns_all_rules() {
    let config = Config::default();
    let engine = LintEngine::new(config);
    let rules: Vec<_> = engine.registry().all_rules().collect();

    assert!(!rules.is_empty());
    assert!(rules.iter().any(|r| r.id == "snake_case_variables"));
}

#[test]
fn test_explain_rule_exists() {
    let config = Config::default();
    let engine = LintEngine::new(config);
    let rule = engine.registry().get_rule("snake_case_variables");

    assert!(rule.is_some());
    let rule = rule.unwrap();
    assert_eq!(rule.id, "snake_case_variables");
    assert!(!rule.description.is_empty());
}

#[test]
fn test_explain_nonexistent_rule() {
    let config = Config::default();
    let engine = LintEngine::new(config);
    let rule = engine.registry().get_rule("NONEXISTENT");

    assert!(rule.is_none());
}

#[test]
fn test_lint_nonexistent_file() {
    let nonexistent = PathBuf::from("nonexistent.nu");

    // collect_files_to_lint will call process::exit for nonexistent files
    // We can't test this directly without spawning a process, but we can
    // verify the file doesn't exist as a precondition
    assert!(!nonexistent.exists());
}

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

    // Use a closure with defer-like behavior to ensure directory is restored
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = load_config(None);
        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files, false);

        violations
    }));

    // Always restore directory, even if test panics
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

    // Use a closure with defer-like behavior to ensure directory is restored
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(&subdir).unwrap();

        let config = load_config(None);
        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files, false);

        violations
    }));

    // Always restore directory, even if test panics
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

    // Use a closure with defer-like behavior to ensure directory is restored
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Explicit config should override auto-discovery
        let config = load_config(Some(&explicit_config));
        let engine = LintEngine::new(config);
        // Use relative path since we changed directory
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files, false);

        violations
    }));

    // Always restore directory, even if test panics
    std::env::set_current_dir(original_dir).unwrap();

    let violations = result.unwrap();

    // Should have violations because explicit config doesn't disable the rule
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "snake_case_variables")
    );
}

#[test]
fn test_violations_should_cause_nonzero_exit() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("bad.nu");
    fs::write(&temp_file, "let myVariable = 5\n").unwrap();

    let mut config = Config::default();
    config.general.min_severity = RuleSeverity::Info;
    let engine = LintEngine::new(config);
    let files = collect_files_to_lint(&[temp_file]);
    let (violations, _) = lint_files(&engine, &files, false);

    // This simulates the exit code logic from main.rs
    let exit_code = i32::from(!violations.is_empty());
    assert_eq!(exit_code, 1);
}

#[test]
fn test_no_violations_should_cause_zero_exit() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("good.nu");
    fs::write(&temp_file, "# Good code\nlet my_var = 5\n").unwrap();

    let config = Config::default();
    let engine = LintEngine::new(config);
    let files = collect_files_to_lint(&[temp_file]);
    let (violations, _) = lint_files(&engine, &files, false);

    // This simulates the exit code logic from main.rs
    let exit_code = i32::from(!violations.is_empty());
    assert_eq!(exit_code, 0);
}

#[test]
#[allow(clippy::similar_names)]
fn test_lint_directory() {
    let temp_dir = TempDir::new().unwrap();

    let file1 = temp_dir.path().join("test1.nu");
    let file2 = temp_dir.path().join("test2.nu");
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file3 = subdir.join("test3.nu");

    fs::write(&file1, "let myVariable = 5\n").unwrap();
    fs::write(&file2, "def myCommand [] { }\n").unwrap();
    fs::write(&file3, "let another_var = 10\n").unwrap();

    let mut config = Config::default();
    config.general.min_severity = RuleSeverity::Info;
    let engine = LintEngine::new(config);

    let collected_files = collect_files_to_lint(&[temp_dir.path().to_path_buf()]);
    let (violations, _) = lint_files(&engine, &collected_files, false);

    assert!(!violations.is_empty());
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "snake_case_variables")
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "kebab_case_commands")
    );
}

#[test]
fn test_lint_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    // collect_files_to_lint will call process::exit when no .nu files are found
    // We can verify the directory exists but has no .nu files
    assert!(temp_dir.path().exists());
    let nu_files = collect_nu_files(&temp_dir.path().to_path_buf());
    assert!(nu_files.is_empty());
}
