use std::{
    borrow::Cow,
    fs,
    io::{self, BufRead},
    path::PathBuf,
    process,
    sync::Mutex,
};

use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use rayon::prelude::*;

use crate::{
    Config, LintEngine, LintLevel, output,
    rules::ALL_RULES,
    sets::{BUILTIN_LINT_SETS, DEFAULT_RULE_MAP},
    violation::Violation,
};

#[derive(Parser)]
#[command(name = "nu-lint")]
#[command(about = "A linter for Nushell scripts", long_about = None)]
#[command(version)]
#[command(
    after_help = "STDIN:\n  If no paths are provided and stdin is not a terminal, nu-lint will \
                  read and lint code from stdin.\n  Example: echo 'let x = 5' | nu-lint\n\n  When \
                  using --fix with stdin, the fixed content will be written to stdout.\n  \
                  Example: echo 'let myVar = 5' | nu-lint --fix"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(help = "Files or directories to lint")]
    pub paths: Vec<PathBuf>,

    #[arg(short, long, help = "Configuration file path")]
    pub config: Option<PathBuf>,

    #[arg(
        short = 'f',
        long = "format",
        alias = "output",
        short_alias = 'o',
        help = "Output format",
        value_enum,
        default_value = "text"
    )]
    pub format: Option<Format>,

    #[arg(long, help = "Apply auto-fixes")]
    pub fix: bool,

    #[arg(long, help = "Show what would be fixed without applying")]
    pub dry_run: bool,

    #[arg(short = 'v', long, help = "Enable verbose debug output")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "List all available rules")]
    ListRules,

    #[command(about = "List all available lint sets")]
    ListSets,

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
    /// VS Code LSP-compatible JSON format
    VscodeJson,
    Github,
}

/// Handle subcommands (list-rules, list-sets, explain)
pub fn handle_command(command: Commands, config: &Config) {
    match command {
        Commands::ListRules => list_rules(config),
        Commands::ListSets => list_sets(),
        Commands::Explain { rule_id } => explain_rule(config, &rule_id),
    }
}

fn is_nushell_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext == "nu")
        || fs::File::open(path)
            .ok()
            .and_then(|file| {
                let mut reader = io::BufReader::new(file);
                let mut first_line = String::new();
                reader.read_line(&mut first_line).ok()?;
                first_line.starts_with("#!").then(|| {
                    first_line
                        .split_whitespace()
                        .any(|word| word.ends_with("/nu") || word == "nu")
                })
            })
            .unwrap_or(false)
}

/// Collect all files to lint from the provided paths, respecting .gitignore
/// files
#[must_use]
pub fn collect_files_to_lint(paths: &[PathBuf]) -> Vec<PathBuf> {
    let (files, errors): (Vec<_>, Vec<_>) = paths
        .iter()
        .map(|path| {
            if !path.exists() {
                return Err(format!("Error: Path not found: {}", path.display()));
            }

            if path.is_file() {
                Ok(if is_nushell_file(path) {
                    vec![path.clone()]
                } else {
                    vec![]
                })
            } else if path.is_dir() {
                let files = collect_nu_files_with_gitignore(path);
                if files.is_empty() {
                    eprintln!("Warning: No .nu files found in {}", path.display());
                }
                Ok(files)
            } else {
                Ok(vec![])
            }
        })
        .partition(Result::is_ok);

    for err in &errors {
        if let Err(msg) = err {
            eprintln!("{msg}");
        }
    }

    let files_to_lint: Vec<PathBuf> = files.into_iter().filter_map(Result::ok).flatten().collect();

    if files_to_lint.is_empty() {
        eprintln!("Error: No files to lint");
        process::exit(2);
    }

    files_to_lint
}

/// Collect .nu files from a directory, respecting .gitignore files
#[must_use]
pub fn collect_nu_files_with_gitignore(dir: &PathBuf) -> Vec<PathBuf> {
    WalkBuilder::new(dir)
        .standard_filters(true)
        .build()
        .filter_map(|result| match result {
            Ok(entry) => {
                let path = entry.path().to_path_buf();
                (path.is_file() && is_nushell_file(&path)).then_some(path)
            }
            Err(err) => {
                eprintln!("Warning: Error walking directory: {err}");
                None
            }
        })
        .collect()
}

/// Lint files in parallel
#[must_use]
pub fn lint_files(engine: &LintEngine, files: &[PathBuf]) -> (Vec<Violation>, bool) {
    let violations_mutex = Mutex::new(Vec::new());
    let errors_mutex = Mutex::new(false);

    files
        .par_iter()
        .for_each(|path| match engine.lint_file(path) {
            Ok(violations) => {
                violations_mutex
                    .lock()
                    .expect("Failed to lock violations mutex")
                    .extend(violations);
            }
            Err(e) => {
                eprintln!("Error linting {}: {}", path.display(), e);
                *errors_mutex.lock().expect("Failed to lock errors mutex") = true;
            }
        });

    let violations = violations_mutex
        .into_inner()
        .expect("Failed to unwrap violations mutex");
    let has_errors = errors_mutex
        .into_inner()
        .expect("Failed to unwrap errors mutex");
    (violations, has_errors)
}

/// Lint content from stdin
#[must_use]
pub fn lint_stdin(engine: &LintEngine, source: &str) -> (Vec<Violation>, bool) {
    let mut violations = engine.lint_str(source);

    // Mark all violations as coming from stdin and store the source
    let stdin_marker: Cow<'static, str> = "<stdin>".to_owned().into();
    let source_content: Cow<'static, str> = source.to_owned().into();

    for violation in &mut violations {
        violation.file = Some(stdin_marker.clone());
        violation.source = Some(source_content.clone());
    }

    (violations, false)
}

/// Format and output linting results
pub fn output_results(violations: &[Violation], format: Option<Format>) {
    let output = match format.unwrap_or(Format::Text) {
        Format::Text | Format::Github => output::format_text(violations),
        Format::Json => output::format_json(violations),
        Format::VscodeJson => output::format_vscode_json(violations),
    };
    println!("{output}");
}

fn list_rules(config: &Config) {
    println!("Available rules:\n");

    for rule in ALL_RULES {
        let lint_level = config.get_lint_level(rule.id);
        println!("{:<40} [{:?}] {}", rule.id, lint_level, rule.explanation);
    }
}

fn list_sets() {
    println!("Available lint sets:\n");

    let mut sorted_sets: Vec<_> = BUILTIN_LINT_SETS.iter().collect();
    sorted_sets.sort_by_key(|(name, _)| *name);

    for (name, set) in sorted_sets {
        println!(
            "{:<20} {} ({} rules)",
            name,
            set.explanation,
            set.rules.len()
        );
    }
}

fn explain_rule(config: &Config, rule_id: &str) {
    if let Some(rule) = ALL_RULES.iter().find(|r| r.id == rule_id) {
        let lint_level = config.get_lint_level(rule.id);
        let default_level = DEFAULT_RULE_MAP
            .rules
            .iter()
            .find(|(id, _)| *id == rule.id)
            .copied()
            .map_or(LintLevel::Warn, |(_, level)| level);
        println!("Rule: {}", rule.id);
        println!("Lint Level: {lint_level:?}");
        println!("Default Lint Level: {default_level}");
        println!("Description: {}", rule.explanation);
    } else {
        eprintln!("Error: Rule '{rule_id}' not found");
        process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env::{current_dir, set_current_dir},
        sync::Mutex,
    };

    use tempfile::TempDir;

    use super::*;
    use crate::config::LintLevel;

    static CHDIR_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_no_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let nu_file_path = temp_dir.path().join("test.nu");

        fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

        let config = Config::default();
        // Default config should be empty but get_lint_level should return defaults
        assert_eq!(config.lints.rules.get("snake_case_variables"), None);
        assert_eq!(
            config.get_lint_level("snake_case_variables"),
            LintLevel::Warn
        );

        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[nu_file_path]);
        let (violations, _) = lint_files(&engine, &files);

        assert!(
            violations
                .iter()
                .any(|v| v.rule_id == "snake_case_variables" && v.lint_level == LintLevel::Warn)
        );
    }

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
        let (violations, _) = lint_files(&engine, &files);

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
            r#"[lints.rules]
        snake_case_variables = "deny""#,
        )
        .unwrap();
        fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

        let original_dir = current_dir().unwrap();

        set_current_dir(temp_dir.path()).unwrap();

        let config = Config::load(None);
        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files);

        set_current_dir(original_dir).unwrap();

        assert!(
            violations
                .iter()
                .any(|v| v.rule_id == "snake_case_variables" && v.lint_level == LintLevel::Deny)
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
            r#"[lints.rules]
        snake_case_variables = "deny""#,
        )
        .unwrap();
        fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

        let original_dir = current_dir().unwrap();

        set_current_dir(&subdir).unwrap();

        let config = Config::load(None);
        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files);

        set_current_dir(original_dir).unwrap();
        assert!(
            violations
                .iter()
                .any(|v| v.rule_id == "snake_case_variables" && v.lint_level == LintLevel::Deny)
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
        fs::write(
            &explicit_config,
            r#"[lints.rules]
        snake_case_variables = "deny""#,
        )
        .unwrap();
        fs::write(&nu_file_path, "let myVariable = 5\n").unwrap();

        let original_dir = current_dir().unwrap();

        set_current_dir(temp_dir.path()).unwrap();

        let config = Config::load(Some(&explicit_config));
        let engine = LintEngine::new(config);
        let files = collect_files_to_lint(&[PathBuf::from("test.nu")]);
        let (violations, _) = lint_files(&engine, &files);

        set_current_dir(original_dir).unwrap();
        assert!(
            violations
                .iter()
                .any(|v| v.rule_id == "snake_case_variables" && v.lint_level == LintLevel::Deny)
        );
    }
}
