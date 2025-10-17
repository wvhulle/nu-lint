use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_lint_file_with_violations() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("bad.nu");
    fs::write(&temp_file, "let myVariable = 5\n").unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg(&temp_file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("warning"))
        .stdout(predicate::str::contains("snake_case_variables"));
}

#[test]
fn test_lint_file_without_violations() {
    // Create a temporary file with no violations
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("good.nu");
    fs::write(&temp_file, "# Good code\nlet my_var = 5\n").unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("No violations found"));
}

#[test]
fn test_list_rules_command() {
    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg("list-rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("snake_case_variables"))
        .stdout(predicate::str::contains("style"))
        .stdout(predicate::str::contains("snake_case"));
}

#[test]
fn test_explain_rule_command() {
    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg("explain")
        .arg("snake_case_variables")
        .assert()
        .success()
        .stdout(predicate::str::contains("snake_case_variables"))
        .stdout(predicate::str::contains("style"))
        .stdout(predicate::str::contains("snake_case"));
}

#[test]
fn test_explain_nonexistent_rule() {
    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg("explain")
        .arg("NONEXISTENT")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_lint_nonexistent_file() {
    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg("nonexistent.nu")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_no_files_specified() {
    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No files specified"));
}

#[test]
fn test_custom_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("custom.toml");
    let nu_file_path = temp_dir.path().join("test.nu");

    // Create a config file
    fs::write(&config_path, "[general]\nmax_severity = \"error\"\n").unwrap();

    // Create a test file with violations
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg("--config")
        .arg(config_path)
        .arg(nu_file_path)
        .assert()
        .failure();
}

#[test]
fn test_auto_discover_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".nu-lint.toml");
    let nu_file_path = temp_dir.path().join("test.nu");

    // Create a .nu-lint.toml config file that disables snake_case_variables
    fs::write(&config_path, "[rules]\nsnake_case_variables = \"off\"\n").unwrap();

    // Create a test file that would normally violate snake_case_variables
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("test.nu")
        .assert()
        .success()
        .stdout(predicate::str::contains("No violations found"));
}

#[test]
fn test_auto_discover_config_in_parent_dir() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".nu-lint.toml");
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let nu_file_path = subdir.join("test.nu");

    // Create a .nu-lint.toml in parent directory that disables snake_case_variables
    fs::write(&config_path, "[rules]\nsnake_case_variables = \"off\"\n").unwrap();

    // Create a test file in subdirectory that would normally violate snake_case_variables
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.current_dir(&subdir)
        .arg("test.nu")
        .assert()
        .success()
        .stdout(predicate::str::contains("No violations found"));
}

#[test]
fn test_explicit_config_overrides_auto_discovery() {
    let temp_dir = TempDir::new().unwrap();
    let auto_config = temp_dir.path().join(".nu-lint.toml");
    let explicit_config = temp_dir.path().join("other.toml");
    let nu_file_path = temp_dir.path().join("test.nu");

    // Auto-discovered config disables snake_case_variables
    fs::write(&auto_config, "[rules]\nsnake_case_variables = \"off\"\n").unwrap();

    // Explicit config enables everything (empty rules)
    fs::write(&explicit_config, "[rules]\n").unwrap();

    // Create a test file that violates snake_case_variables
    fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("--config")
        .arg("other.toml")
        .arg("test.nu")
        .assert()
        .failure()
        .stdout(predicate::str::contains("snake_case_variables"));
}

#[test]
fn test_exit_code_with_violations() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("bad.nu");
    fs::write(&temp_file, "let myVariable = 5\n").unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg(&temp_file).assert().code(1);
}

#[test]
fn test_exit_code_without_violations() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("good.nu");
    fs::write(&temp_file, "# Good code\nlet my_var = 5\n").unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg(&temp_file).assert().code(0);
}

#[test]
fn test_lint_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create some test files with violations
    let file1 = temp_dir.path().join("test1.nu");
    let file2 = temp_dir.path().join("test2.nu");
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file3 = subdir.join("test3.nu");

    fs::write(&file1, "let myVariable = 5\n").unwrap(); // snake_case_variables violation
    fs::write(&file2, "def myCommand [] { }\n").unwrap(); // kebab_case_commands violation
    fs::write(&file3, "let another_var = 10\n").unwrap(); // No violation

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg(temp_dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("snake_case_variables"))
        .stdout(predicate::str::contains("kebab_case_commands"));
}

#[test]
fn test_lint_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();
    cmd.arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No .nu files found"));
}
