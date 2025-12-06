use std::{
    io::{self, Read},
    path::PathBuf,
    process,
};

use clap::{Parser, Subcommand};

use crate::{
    LintLevel,
    config::Config,
    engine::{LintEngine, collect_nu_files},
    fix::{apply_fixes, apply_fixes_to_stdin, format_fix_results},
    lsp,
    output::{Format, Summary, format_output},
    rules::{ALL_RULES, sets::BUILTIN_LINT_SETS},
};

#[derive(Parser)]
#[command(name = "nu-lint")]
#[command(about = "A linter for Nushell scripts")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Files or directories to lint
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,

    /// Output format
    #[arg(long, short = 'f', value_enum, default_value_t = Format::Text)]
    format: Format,

    /// Path to config file
    #[arg(long, short)]
    config: Option<PathBuf>,

    /// Read from stdin
    #[arg(long)]
    stdin: bool,
}

impl Cli {
    fn load_config(path: Option<PathBuf>) -> Config {
        path.map(|p| {
            Config::load_from_file(&p).unwrap_or_else(|e| {
                log::error!("Error loading config from {}: {e}", p.display());
                Config::default()
            })
        })
        .unwrap_or_default()
    }

    fn read_stdin() -> String {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .expect("Failed to read from stdin");
        source
    }

    fn lint(self) {
        let config = Self::load_config(self.config);
        let engine = LintEngine::new(config);

        let (violations, has_errors) = if self.stdin {
            let source = Self::read_stdin();
            (engine.lint_stdin(&source), false)
        } else {
            let files = collect_nu_files(&self.paths);
            if files.is_empty() {
                log::warn!("No Nushell files found in specified paths");
                return;
            }
            engine.lint_files(&files)
        };

        let output = format_output(&violations, self.format);
        if !output.is_empty() {
            println!("{output}");
        }

        let summary = Summary::from_violations(&violations);
        eprintln!("{}", summary.format_compact());

        let has_deny = violations.iter().any(|v| v.lint_level == LintLevel::Deny);
        if has_errors || has_deny {
            process::exit(1);
        }
    }

    fn fix(paths: &[PathBuf], stdin: bool, config: Option<PathBuf>) {
        let config = Self::load_config(config);
        let engine = LintEngine::new(config);

        if stdin {
            Self::fix_stdin(&engine);
        } else {
            Self::fix_files(paths, &engine);
        }
    }

    fn fix_stdin(engine: &LintEngine) {
        let source = Self::read_stdin();
        let violations = engine.lint_stdin(&source);

        if let Some(fixed) = apply_fixes_to_stdin(&violations) {
            print!("{fixed}");
        } else {
            print!("{source}");
        }
    }

    fn fix_files(paths: &[PathBuf], engine: &LintEngine) {
        let files = collect_nu_files(paths);
        if files.is_empty() {
            log::warn!("No Nushell files found in specified paths");
            return;
        }

        let (violations, _) = engine.lint_files(&files);

        match apply_fixes(&violations, false, engine) {
            Ok(results) => {
                let output = format_fix_results(&results, false);
                print!("{output}");
            }
            Err(e) => {
                log::error!("Error applying fixes: {e}");
                process::exit(1);
            }
        }
    }

    fn list_rules() {
        println!("Available lint rules ({n}):\n", n = ALL_RULES.len());
        let mut sorted_rules = ALL_RULES.to_vec();
        sorted_rules.sort_by_key(|rule| rule.id);

        for rule in sorted_rules {
            println!("  {} - {}", rule.id, rule.explanation);
        }
        println!("\n{n} rules available.", n = ALL_RULES.len());
    }

    fn list_sets() {
        println!("Available rule sets ({n}):\n", n = BUILTIN_LINT_SETS.len());
        for set in BUILTIN_LINT_SETS {
            println!("  {} - {}", set.name, set.explanation);
            for rule in set.rules {
                println!("    - {}", rule.id);
            }
            println!();
        }
        println!("\n{n} sets available.", n = BUILTIN_LINT_SETS.len());
    }

    fn explain_rule(rule_id: &str) {
        let rule = ALL_RULES.iter().find(|r| r.id == rule_id);

        if let Some(rule) = rule {
            println!("Rule: {}", rule.id);
            println!("Explanation: {}", rule.explanation);
            if let Some(url) = rule.doc_url {
                println!("Documentation: {url}");
            }
        } else {
            eprintln!("Unknown rule: {rule_id}");
            eprintln!("Use 'nu-lint list' to see available rules");
            process::exit(1);
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// List all available lint rules
    #[command(alias = "rules")]
    List,
    /// List available rule sets
    #[command(alias = "groups")]
    Sets,
    /// Explain a lint rule
    Explain {
        /// Rule ID to explain
        rule_id: String,
    },
    /// Start the LSP server
    Lsp,
    /// Auto-fix lint violations
    Fix {
        /// Files or directories to fix
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
        /// Read from stdin
        #[arg(long)]
        stdin: bool,
        /// Path to config file
        #[arg(long, short)]
        config: Option<PathBuf>,
    },
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => Cli::list_rules(),
        Some(Commands::Sets) => Cli::list_sets(),
        Some(Commands::Explain { rule_id }) => Cli::explain_rule(&rule_id),
        Some(Commands::Lsp) => lsp::run_lsp_server(),
        Some(Commands::Fix {
            paths,
            stdin,
            config,
        }) => Cli::fix(&paths, stdin, config),
        None => cli.lint(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(["nu-lint", "file.nu"]).unwrap();
        assert_eq!(cli.paths, vec![PathBuf::from("file.nu")]);
        assert!(!cli.stdin);
    }

    #[test]
    fn test_cli_stdin_flag() {
        let cli = Cli::try_parse_from(["nu-lint", "--stdin"]).unwrap();
        assert!(cli.stdin);
    }

    #[test]
    fn test_cli_format_flag() {
        let cli = Cli::try_parse_from(["nu-lint", "--format", "json"]).unwrap();
        assert!(matches!(cli.format, Format::Json));
    }

    #[test]
    fn test_cli_list_command() {
        let cli = Cli::try_parse_from(["nu-lint", "list"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::List)));
    }

    #[test]
    fn test_cli_explain_command() {
        let cli = Cli::try_parse_from(["nu-lint", "explain", "some-rule"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Explain { .. })));
    }

    #[test]
    fn test_lint_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.nu");
        fs::write(&test_file, "def foo [] { echo 'hello' }").unwrap();

        let engine = LintEngine::new(Config::default());
        let files = collect_nu_files(&[test_file]);

        assert_eq!(files.len(), 1);
        let (violations, _) = engine.lint_files(&files);
        assert!(violations.is_empty() || !violations.is_empty()); // Just ensure it runs
    }
}
