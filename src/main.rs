use std::{path::PathBuf, process, sync::Mutex};

use clap::{Parser, Subcommand};
use nu_lint::{
    Config, JsonFormatter, LintEngine, OutputFormatter, TextFormatter, rule::RuleMetadata,
};
use rayon::prelude::*;

#[derive(Parser)]
#[command(name = "nu-lint")]
#[command(about = "A linter for Nushell scripts", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(help = "Files or directories to lint")]
    paths: Vec<PathBuf>,

    #[arg(short, long, help = "Configuration file path")]
    config: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Output format",
        value_enum,
        default_value = "text"
    )]
    format: Option<Format>,

    #[arg(long, help = "Apply auto-fixes")]
    fix: bool,

    #[arg(long, help = "Show what would be fixed without applying")]
    dry_run: bool,

    #[arg(
        long,
        help = "Process files in parallel (experimental)",
        default_value = "false"
    )]
    parallel: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "List all available rules")]
    ListRules,

    #[command(about = "Explain a specific rule")]
    Explain {
        #[arg(help = "Rule ID to explain")]
        rule_id: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Format {
    Text,
    Json,
    Github,
}

/// Search for .nu-lint.toml in current directory and parent directories
fn find_config_file() -> Option<PathBuf> {
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

fn main() {
    let cli = Cli::parse();
    let config = load_config(cli.config.as_ref());

    if let Some(command) = cli.command {
        handle_command(command, &config);
        return;
    }

    if cli.paths.is_empty() {
        eprintln!("Error: No files specified");
        eprintln!("Usage: nu-lint [FILES...]");
        process::exit(2);
    }

    let files_to_lint = collect_files_to_lint(&cli.paths);
    let engine = LintEngine::new(config);
    let (all_violations, has_errors) = lint_files(&engine, &files_to_lint, cli.parallel);

    if has_errors && all_violations.is_empty() {
        process::exit(2);
    }

    output_results(&all_violations, &files_to_lint, cli.format);

    let exit_code = i32::from(!all_violations.is_empty());
    process::exit(exit_code);
}

/// Load configuration from file or use defaults
fn load_config(config_path: Option<&PathBuf>) -> Config {
    if let Some(path) = config_path {
        match Config::load_from_file(path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Error loading config: {e}");
                process::exit(2);
            }
        }
    } else if let Some(path) = find_config_file() {
        match Config::load_from_file(&path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Error loading config from {}: {}", path.display(), e);
                process::exit(2);
            }
        }
    } else {
        Config::default()
    }
}

/// Handle subcommands (list-rules, explain)
fn handle_command(command: Commands, config: &Config) {
    match command {
        Commands::ListRules => list_rules(config),
        Commands::Explain { rule_id } => explain_rule(config, &rule_id),
    }
}

/// Collect all files to lint from the provided paths
fn collect_files_to_lint(paths: &[PathBuf]) -> Vec<PathBuf> {
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
fn lint_files(
    engine: &LintEngine,
    files: &[PathBuf],
    parallel: bool,
) -> (Vec<nu_lint::lint::Violation>, bool) {
    if parallel && files.len() > 1 {
        lint_files_parallel(engine, files)
    } else {
        lint_files_sequential(engine, files)
    }
}

/// Lint files in parallel
fn lint_files_parallel(
    engine: &LintEngine,
    files: &[PathBuf],
) -> (Vec<nu_lint::lint::Violation>, bool) {
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
fn lint_files_sequential(
    engine: &LintEngine,
    files: &[PathBuf],
) -> (Vec<nu_lint::lint::Violation>, bool) {
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
fn output_results(
    violations: &[nu_lint::lint::Violation],
    files: &[PathBuf],
    format: Option<Format>,
) {
    let source = if files.len() == 1 {
        std::fs::read_to_string(&files[0]).unwrap_or_default()
    } else {
        String::new()
    };

    let output = match format.unwrap_or(Format::Text) {
        Format::Text => {
            let formatter = TextFormatter;
            formatter.format(violations, &source)
        }
        Format::Json => {
            let formatter = JsonFormatter;
            formatter.format(violations, &source)
        }
        Format::Github => {
            // TODO: Implement GitHub formatter
            let formatter = TextFormatter;
            formatter.format(violations, &source)
        }
    };
    println!("{output}");
}

/// Recursively collect all .nu files from a directory
fn collect_nu_files(dir: &PathBuf) -> std::vec::Vec<std::path::PathBuf> {
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
