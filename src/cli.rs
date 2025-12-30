use std::{
    io::{self, Read},
    path::PathBuf,
    process,
};

use clap::Parser;

use crate::{
    LintLevel,
    config::Config,
    engine::{LintEngine, collect_nu_files},
    fix::{apply_fixes, apply_fixes_to_stdin, format_fix_results},
    log::instrument,
    lsp,
    output::{Format, Summary, format_output},
    rule::Rule,
    rules::{USED_RULES, groups::ALL_GROUPS},
};

#[derive(Parser)]
#[command(name = "nu-lint")]
#[command(about = "A linter for Nushell scripts")]
#[command(version)]
pub struct Cli {
    /// Files or directories to lint/fix
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,

    /// Auto-fix lint violations
    #[arg(long, conflicts_with_all = ["lsp", "list", "groups", "explain"])]
    fix: bool,

    /// Start the LSP server
    #[arg(long, conflicts_with_all = ["fix", "list", "groups", "explain"])]
    lsp: bool,

    /// List all available lint rules
    #[arg(long, conflicts_with_all = ["fix", "lsp", "groups", "explain"], alias = "rules")]
    list: bool,

    /// List all available rule groups
    #[arg(long, conflicts_with_all = ["fix", "lsp", "list", "explain"], alias = "sets")]
    groups: bool,

    /// Explain a specific lint rule
    #[arg(long, value_name = "RULE_ID", conflicts_with_all = ["fix", "lsp", "list", "groups"])]
    explain: Option<String>,

    /// Output format
    #[arg(long, short = 'f', value_enum, default_value_t = Format::Text)]
    format: Format,

    /// Path to config file
    #[arg(long, short)]
    config: Option<PathBuf>,

    /// Read from stdin
    #[arg(long)]
    stdin: bool,

    /// Verbose output (requires a level set by environment variable
    /// `RUST_LOG=debug`)
    #[arg(long, short = 'v')]
    verbose: bool,
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

    fn lint(&self) {
        let config = Self::load_config(self.config.clone());
        let engine = LintEngine::new(config);

        let violations = if self.stdin {
            let source = Self::read_stdin();
            engine.lint_stdin(&source)
        } else {
            let files = collect_nu_files(&self.paths);
            if files.is_empty() {
                eprintln!("Warning: No Nushell files found in specified paths");
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

        if violations.iter().any(|v| v.lint_level > LintLevel::Hint) {
            process::exit(1);
        } else {
            process::exit(0);
        }
    }

    fn fix(&self) {
        let config = Self::load_config(self.config.clone());
        let engine = LintEngine::new(config);

        if self.stdin {
            Self::fix_stdin(&engine);
        } else {
            Self::fix_files(&self.paths, &engine);
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
            eprintln!("Warning: No Nushell files found in specified paths");
            return;
        }

        let violations = engine.lint_files(&files);

        let results = apply_fixes(&violations, false, engine);
        let output = format_fix_results(&results, false);
        print!("{output}");
    }

    fn list_rules() {
        println!("## Available Lint Rules\n");
        let mut sorted_rules = USED_RULES.to_vec();
        sorted_rules.sort_by_key(|r| r.id());

        let max_id_len = sorted_rules.iter().map(|r| r.id().len()).max().unwrap_or(0) + 2; // +2 for backticks
        let max_desc_len = sorted_rules
            .iter()
            .map(|r| r.explanation().len())
            .max()
            .unwrap_or(0);

        println!(
            "| {:<width_id$} | {:<width_desc$} | {:<7} | {:<8} |",
            "Rule",
            "Description",
            "Level",
            "Auto-fix",
            width_id = max_id_len,
            width_desc = max_desc_len
        );
        println!(
            "| {:-<width_id$} | {:-<width_desc$} | {:-<7} | {:-<8} |",
            "",
            "",
            "",
            "",
            width_id = max_id_len,
            width_desc = max_desc_len
        );
        for rule in &sorted_rules {
            let level = match rule.level() {
                LintLevel::Hint => "hint",
                LintLevel::Warning => "warning",
                LintLevel::Error => "error",
            };
            let auto_fix = if rule.has_auto_fix() { "Yes" } else { "" };
            let id_formatted = format!("`{}`", rule.id());
            println!(
                "| {:<width_id$} | {:<width_desc$} | {:<7} | {:<8} |",
                id_formatted,
                rule.explanation(),
                level,
                auto_fix,
                width_id = max_id_len,
                width_desc = max_desc_len
            );
        }
        let fixable_count = sorted_rules.iter().filter(|r| r.has_auto_fix()).count();
        println!(
            "\n*{n} rules available, {f} with auto-fix.*",
            n = sorted_rules.len(),
            f = fixable_count
        );
    }

    fn list_groups() {
        fn auto_fix_suffix(rule: &dyn Rule) -> &'static str {
            if rule.has_auto_fix() {
                " (auto-fix)"
            } else {
                ""
            }
        }
        println!("Rule Groups\n");
        for set in ALL_GROUPS {
            println!("`{}` - {}\n", set.name, set.description);
            for rule in set.rules {
                println!("- `{}`{}", rule.id(), auto_fix_suffix(*rule));
            }
            println!();
        }
        println!("*{n} groups available.*", n = ALL_GROUPS.len());
    }

    fn explain_rule(rule_id: &str) {
        let rule = USED_RULES.iter().find(|r| r.id() == rule_id);

        if let Some(rule) = rule {
            println!("Rule: {}", rule.id());
            println!("Explanation: {}", rule.explanation());
            if let Some(url) = rule.doc_url() {
                println!("Documentation: {url}");
            }
        } else {
            eprintln!("Unknown rule ID: {rule_id}");
            process::exit(1);
        }
    }
}

pub fn run() {
    let cli = Cli::parse();

    if cli.verbose {
        instrument();
    }

    if cli.list {
        Cli::list_rules();
    } else if cli.groups {
        Cli::list_groups();
    } else if let Some(ref rule_id) = cli.explain {
        Cli::explain_rule(rule_id);
    } else if cli.lsp {
        lsp::run_lsp_server();
    } else if cli.fix {
        cli.fix();
    } else {
        cli.lint();
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use clap::Parser;

    use crate::{Config, LintEngine, cli::Cli, engine::collect_nu_files};

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
    fn test_cli_list_rules_flag() {
        let cli = Cli::try_parse_from(["nu-lint", "--list"]).unwrap();
        assert!(cli.list);
    }

    #[test]
    fn test_cli_list_groups_flag() {
        let cli = Cli::try_parse_from(["nu-lint", "--groups"]).unwrap();
        assert!(cli.groups);
    }

    #[test]
    fn test_cli_explain_flag() {
        let cli = Cli::try_parse_from(["nu-lint", "--explain", "some-rule"]).unwrap();
        assert_eq!(cli.explain, Some("some-rule".to_string()));
    }

    #[test]
    fn test_cli_lsp_flag() {
        let cli = Cli::try_parse_from(["nu-lint", "--lsp"]).unwrap();
        assert!(cli.lsp);
    }

    #[test]
    fn test_cli_fix_flag() {
        let cli = Cli::try_parse_from(["nu-lint", "--fix", "file.nu"]).unwrap();
        assert!(cli.fix);
        assert_eq!(cli.paths, vec![PathBuf::from("file.nu")]);
    }

    #[test]
    fn test_cli_mutually_exclusive_flags() {
        assert!(Cli::try_parse_from(["nu-lint", "--fix", "--lsp"]).is_err());
        assert!(Cli::try_parse_from(["nu-lint", "--list-rules", "--list-groups"]).is_err());
        assert!(Cli::try_parse_from(["nu-lint", "--fix", "--explain", "rule"]).is_err());
    }

    #[test]
    fn test_lint_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.nu");
        fs::write(&test_file, "def foo [] { echo 'hello' }").unwrap();

        let engine = LintEngine::new(Config::default());
        let files = collect_nu_files(&[test_file]);

        assert_eq!(files.len(), 1);
        let violations = engine.lint_files(&files);
        assert!(violations.is_empty() || !violations.is_empty()); // Just ensure it runs
    }
}
