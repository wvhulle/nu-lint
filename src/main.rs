use clap::{Parser, Subcommand};
use nu_lint::{Config, LintEngine, OutputFormatter, TextFormatter};
use std::path::PathBuf;
use std::process;

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

    let config = if let Some(config_path) = cli.config {
        match Config::load_from_file(&config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                process::exit(2);
            }
        }
    } else if let Some(config_path) = find_config_file() {
        match Config::load_from_file(&config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Error loading config from {}: {}", config_path.display(), e);
                process::exit(2);
            }
        }
    } else {
        Config::default()
    };

    if let Some(command) = cli.command {
        match command {
            Commands::ListRules => {
                list_rules(&config);
                return;
            }
            Commands::Explain { rule_id } => {
                explain_rule(&config, &rule_id);
                return;
            }
        }
    }

    if cli.paths.is_empty() {
        eprintln!("Error: No files specified");
        eprintln!("Usage: nu-lint [FILES...]");
        process::exit(2);
    }

    let engine = LintEngine::new(config);
    let mut all_violations = Vec::new();
    let mut has_errors = false;
    let mut files_to_lint = Vec::new();

    for path in &cli.paths {
        if !path.exists() {
            eprintln!("Error: Path not found: {}", path.display());
            has_errors = true;
            continue;
        }

        if path.is_file() {
            files_to_lint.push(path.clone());
        } else if path.is_dir() {
            match collect_nu_files(path) {
                Ok(files) => {
                    if files.is_empty() {
                        eprintln!("Warning: No .nu files found in {}", path.display());
                    }
                    files_to_lint.extend(files);
                }
                Err(e) => {
                    eprintln!("Error scanning directory {}: {}", path.display(), e);
                    has_errors = true;
                }
            }
        }
    }

    if files_to_lint.is_empty() {
        if !has_errors {
            eprintln!("Error: No files to lint");
        }
        process::exit(2);
    }

    for path in &files_to_lint {
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

    if has_errors && all_violations.is_empty() {
        process::exit(2);
    }

    let source = if files_to_lint.len() == 1 {
        std::fs::read_to_string(&files_to_lint[0]).unwrap_or_default()
    } else {
        String::new()
    };

    let formatter = TextFormatter;
    let output = formatter.format(&all_violations, &source);
    println!("{}", output);

    let exit_code = if all_violations.is_empty() { 0 } else { 1 };

    process::exit(exit_code);
}

/// Recursively collect all .nu files from a directory
fn collect_nu_files(dir: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    let mut nu_files = Vec::new();
    visit_dir(dir, &mut nu_files);
    Ok(nu_files)
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
        eprintln!("Error: Rule '{}' not found", rule_id);
        process::exit(2);
    }
}
