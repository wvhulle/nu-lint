mod common;

use std::fs;

use nu_lint::cli::{collect_files_to_lint, lint_files};
use nu_lint::config::{Config, RuleSeverity};
use nu_lint::engine::LintEngine;
use tempfile::TempDir;

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
