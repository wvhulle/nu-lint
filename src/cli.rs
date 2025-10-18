use std::{path::PathBuf, process, sync::Mutex};

use crate::{
    Config, JsonFormatter, LintEngine, OutputFormatter, TextFormatter, lint::Violation,
    rule::RuleMetadata,
};
use clap::{Parser, Subcommand};
use rayon::prelude::*;

#[cfg(test)]
use assert_cmd;
#[cfg(test)]
use predicates;
#[cfg(test)]
use tempfile;

#[derive(Parser)]
#[command(name = "nu-lint")]
#[command(about = "A linter for Nushell scripts", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(help = "Files or directories to lint")]
    pub paths: Vec<PathBuf>,

    #[arg(short, long, help = "Configuration file path")]
    pub config: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Output format",
        value_enum,
        default_value = "text"
    )]
    pub format: Option<Format>,

    #[arg(long, help = "Apply auto-fixes")]
    pub fix: bool,

    #[arg(long, help = "Show what would be fixed without applying")]
    pub dry_run: bool,

    #[arg(
        long,
        help = "Process files in parallel (experimental)",
        default_value = "false"
    )]
    pub parallel: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "List all available rules")]
    ListRules,

    #[command(about = "Explain a specific rule")]
    Explain {
        #[arg(help = "Rule ID to explain")]
        rule_id: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum Format {
    Text,
    Json,
    Github,
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

/// Handle subcommands (list-rules, explain)
pub fn handle_command(command: Commands, config: &Config) {
    match command {
        Commands::ListRules => list_rules(config),
        Commands::Explain { rule_id } => explain_rule(config, &rule_id),
    }
}

/// Collect all files to lint from the provided paths
#[must_use]
pub fn collect_files_to_lint(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files_to_lint = Vec::new();
    let mut has_errors = false;

    for path in paths {
        if !path.exists() {
            eprintln!("Error: Path not found: {}", path.display());
            has_errors = true;
            continue;
        }

        if path.is_file() {
            files_to_lint.push(path.clone());
        } else if path.is_dir() {
            let files = collect_nu_files(path);
            if files.is_empty() {
                eprintln!("Warning: No .nu files found in {}", path.display());
            }
            files_to_lint.extend(files);
        }
    }

    if files_to_lint.is_empty() {
        if !has_errors {
            eprintln!("Error: No files to lint");
        }
        process::exit(2);
    }

    files_to_lint
}

/// Lint files either in parallel or sequentially
#[must_use]
pub fn lint_files(
    engine: &LintEngine,
    files: &[PathBuf],
    parallel: bool,
) -> (Vec<Violation>, bool) {
    if parallel && files.len() > 1 {
        lint_files_parallel(engine, files)
    } else {
        lint_files_sequential(engine, files)
    }
}

/// Lint files in parallel
fn lint_files_parallel(engine: &LintEngine, files: &[PathBuf]) -> (Vec<Violation>, bool) {
    let violations_mutex = Mutex::new(Vec::new());
    let errors_mutex = Mutex::new(false);

    files
        .par_iter()
        .for_each(|path| match engine.lint_file(path) {
            Ok(violations) => {
                let mut all_viols = violations_mutex.lock().unwrap();
                all_viols.extend(violations);
            }
            Err(e) => {
                eprintln!("Error linting {}: {}", path.display(), e);
                let mut has_errs = errors_mutex.lock().unwrap();
                *has_errs = true;
            }
        });

    let violations = violations_mutex.into_inner().unwrap();
    let has_errors = errors_mutex.into_inner().unwrap();
    (violations, has_errors)
}

/// Lint files sequentially
fn lint_files_sequential(engine: &LintEngine, files: &[PathBuf]) -> (Vec<Violation>, bool) {
    let mut all_violations = Vec::new();
    let mut has_errors = false;

    for path in files {
        match engine.lint_file(path) {
            Ok(violations) => {
                all_violations.extend(violations);
            }
            Err(e) => {
                eprintln!("Error linting {}: {}", path.display(), e);
                has_errors = true;
            }
        }
    }

    (all_violations, has_errors)
}

/// Format and output linting results
pub fn output_results(violations: &[Violation], files: &[PathBuf], format: Option<Format>) {
    let source = if files.len() == 1 {
        std::fs::read_to_string(&files[0]).unwrap_or_default()
    } else {
        String::new()
    };

    let output = match format.unwrap_or(Format::Text) {
        Format::Text | Format::Github => TextFormatter.format(violations, &source),
        Format::Json => JsonFormatter.format(violations, &source),
    };
    println!("{output}");
}

/// Recursively collect all .nu files from a directory
fn collect_nu_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut nu_files = Vec::new();
    visit_dir(dir, &mut nu_files);
    nu_files
}

fn visit_dir(dir: &PathBuf, nu_files: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Warning: Cannot read directory {}: {}", dir.display(), e);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Warning: Cannot read entry in {}: {}", dir.display(), e);
                continue;
            }
        };

        let path = entry.path();

        if path.is_dir() {
            visit_dir(&path, nu_files);
        } else if path.extension().and_then(|s| s.to_str()) == Some("nu") {
            nu_files.push(path);
        }
    }
}

fn list_rules(config: &Config) {
    let engine = LintEngine::new(config.clone());
    println!("Available rules:\n");

    for rule in engine.registry().all_rules() {
        println!(
            "{:<8} [{:<12}] {} - {}",
            rule.id(),
            rule.category(),
            rule.severity(),
            rule.description()
        );
    }
}

fn explain_rule(config: &Config, rule_id: &str) {
    let engine = LintEngine::new(config.clone());

    if let Some(rule) = engine.registry().get_rule(rule_id) {
        println!("Rule: {}", rule.id());
        println!("Category: {}", rule.category());
        println!("Severity: {}", rule.severity());
        println!("Description: {}", rule.description());
    } else {
        eprintln!("Error: Rule '{rule_id}' not found");
        process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Use a static mutex to prevent race conditions with tests that change directories
    static CHDIR_MUTEX: Mutex<()> = Mutex::new(());

    fn with_temp_dir<F>(f: F)
    where
        F: FnOnce(&TempDir),
    {
        let _guard = CHDIR_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            f(&temp_dir);
            let _ = std::env::set_current_dir(original_dir);
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
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".nu-lint.toml");
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(&config_path, "[rules]\n").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        if std::env::set_current_dir(&subdir).is_ok() {
            let found = find_config_file();
            assert!(found.is_some());
            assert_eq!(found.unwrap(), config_path);
            let _ = std::env::set_current_dir(original_dir);
        } else {
            // Skip test if we can't change directory
        }
    }

    #[test]
    fn test_find_config_file_not_found() {
        with_temp_dir(|_temp_dir| {
            let found = find_config_file();
            assert!(found.is_none());
        });
    }

    #[test]
    fn test_collect_files_to_lint_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.nu");
        fs::write(&file_path, "let x = 5\n").unwrap();

        let files = collect_files_to_lint(std::slice::from_ref(&file_path));
        assert_eq!(files, vec![file_path]);
    }

    #[test]
    fn test_collect_files_to_lint_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test1.nu");
        let file2 = temp_dir.path().join("test2.nu");
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file3 = subdir.join("test3.nu");

        fs::write(&file1, "let x = 5\n").unwrap();
        fs::write(&file2, "let y = 10\n").unwrap();
        fs::write(&file3, "let z = 15\n").unwrap();

        let collected_files = collect_files_to_lint(&[temp_dir.path().to_path_buf()]);
        assert_eq!(collected_files.len(), 3);
        assert!(collected_files.contains(&file1));
        assert!(collected_files.contains(&file2));
        assert!(collected_files.contains(&file3));
    }

    #[test]
    fn test_collect_nu_files() {
        let temp_dir = TempDir::new().unwrap();
        let nu_file = temp_dir.path().join("test.nu");
        let other_file = temp_dir.path().join("test.txt");
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let nu_file_in_subdir = subdir.join("nested.nu");

        fs::write(&nu_file, "let x = 5\n").unwrap();
        fs::write(&other_file, "not a nu file\n").unwrap();
        fs::write(&nu_file_in_subdir, "let y = 10\n").unwrap();

        let files = collect_nu_files(&temp_dir.path().to_path_buf());
        assert_eq!(files.len(), 2);
        assert!(files.contains(&nu_file));
        assert!(files.contains(&nu_file_in_subdir));
        assert!(!files.contains(&other_file));
    }

    #[test]
    fn test_load_config_with_explicit_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\nmax_severity = \"error\"\n").unwrap();

        let config = load_config(Some(&config_path));
        assert_eq!(
            config.general.max_severity,
            crate::config::RuleSeverity::Error
        );
    }

    #[test]
    fn test_load_config_auto_discover() {
        let _guard = CHDIR_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".nu-lint.toml");
        fs::write(&config_path, "[general]\nmax_severity = \"warning\"\n").unwrap();

        // Store original directory
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory for this test only
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test auto-discovery
        let config = load_config(None);
        assert_eq!(
            config.general.max_severity,
            crate::config::RuleSeverity::Warning
        );

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_load_config_default() {
        with_temp_dir(|_temp_dir| {
            let config = load_config(None);
            assert_eq!(config, Config::default());
        });
    }

    mod integration_tests {
        use super::*;
        use assert_cmd::Command as AssertCommand;
        use predicates::prelude::*;

        #[test]
        fn test_lint_file_with_violations() {
            let temp_dir = TempDir::new().unwrap();
            let temp_file = temp_dir.path().join("bad.nu");
            fs::write(&temp_file, "let myVariable = 5\n").unwrap();

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg(&temp_file)
                .assert()
                .failure()
                .stdout(predicate::str::contains("warning"))
                .stdout(predicate::str::contains("snake_case_variables"));
        }

        #[test]
        fn test_lint_file_without_violations() {
            let temp_dir = TempDir::new().unwrap();
            let temp_file = temp_dir.path().join("good.nu");
            fs::write(&temp_file, "# Good code\nlet my_var = 5\n").unwrap();

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg(&temp_file)
                .assert()
                .success()
                .stdout(predicate::str::contains("No violations found"));
        }

        #[test]
        fn test_list_rules_command() {
            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg("list-rules")
                .assert()
                .success()
                .stdout(predicate::str::contains("snake_case_variables"))
                .stdout(predicate::str::contains("style"))
                .stdout(predicate::str::contains("snake_case"));
        }

        #[test]
        fn test_explain_rule_command() {
            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
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
            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg("explain")
                .arg("NONEXISTENT")
                .assert()
                .failure()
                .stderr(predicate::str::contains("not found"));
        }

        #[test]
        fn test_lint_nonexistent_file() {
            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg("nonexistent.nu")
                .assert()
                .failure()
                .stderr(predicate::str::contains("not found"));
        }

        #[test]
        fn test_no_files_specified() {
            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.assert()
                .failure()
                .stderr(predicate::str::contains("No files specified"));
        }

        #[test]
        fn test_custom_config_file() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("custom.toml");
            let nu_file_path = temp_dir.path().join("test.nu");

            fs::write(&config_path, "[general]\nmax_severity = \"error\"\n").unwrap();
            fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
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

            fs::write(&config_path, "[rules]\nsnake_case_variables = \"off\"\n").unwrap();
            fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
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

            fs::write(&config_path, "[rules]\nsnake_case_variables = \"off\"\n").unwrap();
            fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
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

            fs::write(&auto_config, "[rules]\nsnake_case_variables = \"off\"\n").unwrap();
            fs::write(&explicit_config, "[rules]\n").unwrap();
            fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
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

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg(&temp_file).assert().code(1);
        }

        #[test]
        fn test_exit_code_without_violations() {
            let temp_dir = TempDir::new().unwrap();
            let temp_file = temp_dir.path().join("good.nu");
            fs::write(&temp_file, "# Good code\nlet my_var = 5\n").unwrap();

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg(&temp_file).assert().code(0);
        }

        #[test]
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

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg(temp_dir.path())
                .assert()
                .failure()
                .stdout(predicate::str::contains("snake_case_variables"))
                .stdout(predicate::str::contains("kebab_case_commands"));
        }

        #[test]
        fn test_lint_empty_directory() {
            let temp_dir = TempDir::new().unwrap();

            let mut cmd = AssertCommand::cargo_bin("nu-lint").unwrap();
            cmd.arg(temp_dir.path())
                .assert()
                .failure()
                .stderr(predicate::str::contains("No .nu files found"));
        }
    }
}
