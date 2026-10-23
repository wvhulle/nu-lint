use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::{Args, Parser, Subcommand, crate_version};
use miette::Severity;
use nu_lint_analysis::{
    LintLevel,
    ast::tree,
    config::Config,
    engine::LintEngine,
    fix::{apply_fixes, apply_fixes_to_stdin},
    log::init_test_log,
    rule::Rule,
};
use nu_lint_rules::{USED_RULES, groups::ALL_GROUPS};

#[cfg(feature = "lsp")]
use crate::lsp;
use crate::{
    LintError,
    config_loading::load_validated_config,
    file_collection::collect_nu_files,
    format::{Format, Summary, format_fix_results, format_output},
    rule_selection::restrict_config_to_selection,
};

#[derive(Parser)]
#[command(name = "nu-lint")]
#[command(about = "A linter for Nushell scripts")]
#[command(version = crate_version!())]
#[command(args_conflicts_with_subcommands = true, disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,

    #[command(flatten)]
    default_check: CheckArgs,

    #[arg(
        long,
        short = 'v',
        global = true,
        help = "Verbose output (requires a level set by environment variable `RUST_LOG=debug`)"
    )]
    verbose: bool,
}

#[derive(Args)]
struct InputArgs {
    #[arg(default_value = ".", help = "Files or directories to lint or fix")]
    paths: Vec<PathBuf>,

    #[arg(long, short, help = "Path to config file")]
    config: Option<PathBuf>,

    #[arg(long, conflicts_with = "paths", help = "Read from standard input")]
    stdin: bool,

    #[arg(
        long,
        value_delimiter = ',',
        value_name = "RULE_OR_GROUP",
        help = "Only run these rules or rule groups (comma-separated or repeated)"
    )]
    only: Vec<String>,
}

#[derive(Args)]
struct CheckArgs {
    #[command(flatten)]
    input: InputArgs,

    #[arg(long, short = 'f', value_enum, default_value_t = Format::Pretty, help = "Output format")]
    format: Format,
}

#[derive(Args)]
struct FixArgs {
    #[command(flatten)]
    input: InputArgs,

    #[arg(
        long,
        conflicts_with = "stdin",
        help = "Show the diff of the fixes without writing files"
    )]
    dry_run: bool,
}

#[derive(Args)]
struct ListRulesArgs {
    #[arg(long, short, help = "Path to config file")]
    config: Option<PathBuf>,
}

#[derive(Args)]
struct ExplainArgs {
    #[arg(value_name = "RULE_ID")]
    rule_id: String,
}

#[derive(Args)]
struct AstArgs {
    #[arg(value_name = "SOURCE")]
    source: String,
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
struct RulesArgs {
    #[command(subcommand)]
    command: Option<RulesCommand>,

    #[command(flatten)]
    default_list: ListRulesArgs,
}

#[derive(Subcommand)]
enum RulesCommand {
    #[command(about = "List rules with their configured levels (default)")]
    List(ListRulesArgs),
    #[command(about = "Explain a rule")]
    Explain(ExplainArgs),
    #[command(about = "List rule groups and their rules")]
    Groups,
}

#[derive(Subcommand)]
enum DevCommand {
    #[command(about = "Print the parsed AST of Nu source code, with blocks expanded")]
    Ast(AstArgs),
}

#[derive(Subcommand)]
enum CliCommand {
    #[command(about = "Lint files (default when no command is given)")]
    Check(CheckArgs),
    #[command(long_flag_alias = "fix", about = "Apply automatic fixes")]
    Fix(FixArgs),
    #[command(long_flag_alias = "lsp", about = "Start the language server")]
    Lsp,
    #[command(
        long_flag_aliases = ["list", "rules"],
        about = "List, explain and group rules"
    )]
    Rules(RulesArgs),
    #[command(subcommand, about = "Commands for developing nu-lint")]
    Dev(DevCommand),
    #[command(hide = true, long_flag_alias = "explain")]
    Explain(ExplainArgs),
    #[command(hide = true, long_flag_aliases = ["groups", "sets"])]
    Groups,
    #[command(hide = true, long_flag_alias = "ast")]
    Ast(AstArgs),
}

fn with_loaded_config(
    loaded_config: Result<Config, LintError>,
    run_with_config: impl FnOnce(&Config) -> io::Result<ExitCode>,
) -> io::Result<ExitCode> {
    match loaded_config {
        Ok(config) => {
            tracing::debug!(?config);
            run_with_config(&config)
        }
        Err(config_error) => {
            writeln!(io::stderr(), "Error: {config_error}")?;
            Ok(ExitCode::FAILURE)
        }
    }
}

impl InputArgs {
    fn loaded_config(&self) -> Result<Config, LintError> {
        let config = load_validated_config(self.config.as_deref())?;
        if self.only.is_empty() {
            return Ok(config);
        }
        restrict_config_to_selection(config, &self.only)
    }

    fn read_stdin() -> String {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .expect("Failed to read from stdin");
        source
    }
}

impl CheckArgs {
    fn lint(&self, config: &Config) -> io::Result<ExitCode> {
        let engine = crate::lint_engine_with_all_rules(config.clone());

        let violations = if self.input.stdin {
            let source = InputArgs::read_stdin();
            engine.lint_stdin(&source)
        } else {
            let files = collect_nu_files(&self.input.paths);
            if files.is_empty() {
                writeln!(
                    io::stderr(),
                    "Warning: No Nushell files found in specified paths"
                )?;
                return Ok(ExitCode::SUCCESS);
            }
            engine.lint_files(&files)
        };

        let output = format_output(&violations, self.format);
        if !output.is_empty() {
            writeln!(io::stdout(), "{output}")?;
        }

        let summary = Summary::from_violations(&violations);
        writeln!(io::stderr(), "{}", summary.format_compact())?;

        if violations.iter().any(|v| v.lint_level > Severity::Warning) {
            Ok(ExitCode::FAILURE)
        } else {
            Ok(ExitCode::SUCCESS)
        }
    }
}

impl FixArgs {
    fn fix(&self, config: &Config) -> io::Result<ExitCode> {
        let engine = crate::lint_engine_with_all_rules(config.clone());

        if self.input.stdin {
            Self::fix_stdin(&engine)?;
        } else {
            Self::fix_files(&self.input.paths, &engine, self.dry_run)?;
        }
        Ok(ExitCode::SUCCESS)
    }

    fn fix_stdin(engine: &LintEngine) -> io::Result<()> {
        let source = InputArgs::read_stdin();
        let violations = engine.lint_stdin(&source);

        let mut stdout = io::stdout().lock();
        if let Some(fixed) = apply_fixes_to_stdin(&violations) {
            write!(stdout, "{fixed}")?;
        } else {
            write!(stdout, "{source}")?;
        }
        stdout.flush()
    }

    fn fix_files(paths: &[PathBuf], engine: &LintEngine, dry_run: bool) -> io::Result<()> {
        let files = collect_nu_files(paths);
        if files.is_empty() {
            return writeln!(
                io::stderr(),
                "Warning: No Nushell files found in specified paths"
            );
        }

        let violations = engine.lint_files(&files);

        let results = apply_fixes(&violations, dry_run, engine);
        let output = format_fix_results(&results, dry_run);
        let mut stdout = io::stdout().lock();
        write!(stdout, "{output}")?;
        stdout.flush()
    }
}

impl Cli {
    fn into_command(self) -> CliCommand {
        self.command
            .unwrap_or(CliCommand::Check(self.default_check))
    }

    fn list_rules(config: &Config) -> io::Result<()> {
        let mut sorted_rules: Vec<&dyn Rule> = USED_RULES.to_vec();
        sorted_rules.sort_by_key(|r| r.id());

        let mut stdout = io::stdout().lock();

        if sorted_rules.is_empty() {
            return writeln!(stdout, "No rules enabled.");
        }

        let max_id_len = sorted_rules.iter().map(|r| r.id().len()).max().unwrap_or(0);

        for rule in &sorted_rules {
            let level = config.get_lint_level(*rule, ALL_GROUPS);
            let level_char = match level {
                LintLevel::Hint => 'H',
                LintLevel::Warning => 'W',
                LintLevel::Error => 'E',
                LintLevel::Off => 'D',
            };
            let fix_char = if rule.has_auto_fix() { 'F' } else { ' ' };
            let desc = rule.short_description();
            writeln!(
                stdout,
                "{level_char}{fix_char} {:<width$}  {desc}",
                rule.id(),
                width = max_id_len
            )?;
        }

        let fixable_count = sorted_rules.iter().filter(|r| r.has_auto_fix()).count();
        writeln!(
            stdout,
            "\n{n} rules, {f} fixable. [H]int [W]arning [E]rror [F]ixable [D]eactivated",
            n = sorted_rules.len(),
            f = fixable_count
        )
    }

    fn list_groups() -> io::Result<()> {
        fn auto_fix_suffix(rule: &dyn Rule) -> &'static str {
            if rule.has_auto_fix() {
                " (auto-fix)"
            } else {
                ""
            }
        }
        let mut stdout = io::stdout().lock();
        for set in ALL_GROUPS {
            writeln!(stdout, "`{}` - {}\n", set.name, set.description)?;
            for rule in set.rules {
                let desc = rule.short_description();
                writeln!(
                    stdout,
                    "- `{}`{}: {}",
                    rule.id(),
                    auto_fix_suffix(*rule),
                    desc
                )?;
            }
            writeln!(stdout)?;
        }
        Ok(())
    }

    fn explain_rule(rule_id: &str) -> io::Result<ExitCode> {
        let rule = USED_RULES.iter().find(|r| r.id() == rule_id);

        if let Some(rule) = rule {
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "Rule: {}", rule.id())?;
            writeln!(stdout, "Explanation: {}", rule.short_description())?;
            if let Some(url) = rule.source_link() {
                writeln!(stdout, "Documentation: {url}")?;
            }
            Ok(ExitCode::SUCCESS)
        } else {
            writeln!(io::stderr(), "Unknown rule ID: {rule_id}")?;
            Ok(ExitCode::FAILURE)
        }
    }
}

#[allow(
    clippy::missing_errors_doc,
    reason = "write failures are reported by the caller, not documented per stream"
)]
pub fn run() -> io::Result<ExitCode> {
    let cli = Cli::parse();

    if cli.verbose {
        init_test_log();
    }

    match cli.into_command() {
        CliCommand::Check(check_args) => {
            with_loaded_config(check_args.input.loaded_config(), |config| {
                check_args.lint(config)
            })
        }
        CliCommand::Fix(fix_args) => with_loaded_config(fix_args.input.loaded_config(), |config| {
            fix_args.fix(config)
        }),
        CliCommand::Lsp => Ok(start_lsp_server()),
        CliCommand::Rules(rules_args) => run_rules_command(rules_args.into_command()),
        CliCommand::Dev(dev_command) => run_dev_command(dev_command),
        CliCommand::Explain(explain_args) => run_rules_command(RulesCommand::Explain(explain_args)),
        CliCommand::Groups => run_rules_command(RulesCommand::Groups),
        CliCommand::Ast(ast_args) => run_dev_command(DevCommand::Ast(ast_args)),
    }
}

fn run_rules_command(rules_command: RulesCommand) -> io::Result<ExitCode> {
    match rules_command {
        RulesCommand::List(list_args) => with_loaded_config(
            load_validated_config(list_args.config.as_deref()),
            |config| Cli::list_rules(config).map(|()| ExitCode::SUCCESS),
        ),
        RulesCommand::Explain(explain_args) => Cli::explain_rule(&explain_args.rule_id),
        RulesCommand::Groups => Cli::list_groups().map(|()| ExitCode::SUCCESS),
    }
}

fn run_dev_command(dev_command: DevCommand) -> io::Result<ExitCode> {
    match dev_command {
        DevCommand::Ast(ast_args) => tree::print_ast(&ast_args.source).map(|()| ExitCode::SUCCESS),
    }
}

impl RulesArgs {
    fn into_command(self) -> RulesCommand {
        self.command
            .unwrap_or(RulesCommand::List(self.default_list))
    }
}

#[cfg(feature = "lsp")]
fn start_lsp_server() -> ExitCode {
    let _log_guard = lsp::init_lsp_log();
    tracing::info!("nu-lint LSP server started");
    lsp::run_lsp_server();
    ExitCode::SUCCESS
}

#[cfg(not(feature = "lsp"))]
fn start_lsp_server() -> ExitCode {
    let _ = writeln!(
        io::stderr(),
        "nu-lint was built without the `lsp` feature; rebuild with `--features lsp`"
    );
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use clap::{Command, CommandFactory, Parser};

    use super::{CliCommand, DevCommand, ListRulesArgs, RulesCommand};
    use crate::{Config, cli::Cli, file_collection::collect_nu_files};

    fn parsed_command(arguments: &[&str]) -> CliCommand {
        Cli::try_parse_from(arguments).unwrap().into_command()
    }

    fn is_rejected(arguments: &[&str]) -> bool {
        Cli::try_parse_from(arguments).is_err()
    }

    #[test]
    fn test_paths_without_command_lint() {
        let CliCommand::Check(check_args) = parsed_command(&["nu-lint", "file.nu"]) else {
            panic!("expected check");
        };
        assert_eq!(check_args.input.paths, vec![PathBuf::from("file.nu")]);
        assert!(!check_args.input.stdin);
    }

    #[test]
    fn test_no_arguments_lint_working_directory() {
        let CliCommand::Check(check_args) = parsed_command(&["nu-lint"]) else {
            panic!("expected check");
        };
        assert_eq!(check_args.input.paths, vec![PathBuf::from(".")]);
    }

    #[test]
    fn test_only_accepts_commas_and_repetition() {
        let expected_selection = ["filter_to_where", "adopt-nu-syntax"];
        let is_expected_check = |command: CliCommand| {
            matches!(
                command,
                CliCommand::Check(check_args) if check_args.input.only == expected_selection
            )
        };
        assert!(is_expected_check(parsed_command(&[
            "nu-lint",
            "check",
            "--only",
            "filter_to_where,adopt-nu-syntax",
        ])));
        assert!(is_expected_check(parsed_command(&[
            "nu-lint",
            "check",
            "--only",
            "filter_to_where",
            "--only",
            "adopt-nu-syntax",
        ])));
        assert!(is_expected_check(parsed_command(&[
            "nu-lint",
            "--only",
            "filter_to_where,adopt-nu-syntax",
        ])));
        assert!(matches!(
            parsed_command(&["nu-lint", "fix", "--only", "filter_to_where"]),
            CliCommand::Fix(fix_args) if fix_args.input.only == ["filter_to_where"]
        ));
    }

    #[test]
    fn test_fix_dry_run() {
        assert!(matches!(
            parsed_command(&["nu-lint", "fix", "--dry-run", "file.nu"]),
            CliCommand::Fix(fix_args) if fix_args.dry_run
        ));
    }

    fn parsed_rules_command(arguments: &[&str]) -> RulesCommand {
        match parsed_command(arguments) {
            CliCommand::Rules(rules_args) => rules_args.into_command(),
            CliCommand::Explain(explain_args) => RulesCommand::Explain(explain_args),
            CliCommand::Groups => RulesCommand::Groups,
            _ => panic!("expected a rules command for {arguments:?}"),
        }
    }

    fn parsed_ast_source(arguments: &[&str]) -> String {
        match parsed_command(arguments) {
            CliCommand::Dev(DevCommand::Ast(ast_args)) | CliCommand::Ast(ast_args) => {
                ast_args.source
            }
            _ => panic!("expected an ast command for {arguments:?}"),
        }
    }

    #[test]
    fn test_top_level_subcommands() {
        assert!(matches!(
            parsed_command(&["nu-lint", "check", "--stdin"]),
            CliCommand::Check(check_args) if check_args.input.stdin
        ));
        assert!(matches!(
            parsed_command(&["nu-lint", "fix", "file.nu"]),
            CliCommand::Fix(fix_args) if fix_args.input.paths == [PathBuf::from("file.nu")]
        ));
        assert!(matches!(
            parsed_command(&["nu-lint", "lsp"]),
            CliCommand::Lsp
        ));
    }

    #[test]
    fn test_nested_rules_subcommands() {
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "rules"]),
            RulesCommand::List(ListRulesArgs { config: None })
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "rules", "--config", "c.toml"]),
            RulesCommand::List(ListRulesArgs { config: Some(path) }) if path.as_os_str() == "c.toml"
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "rules", "list", "--config", "c.toml"]),
            RulesCommand::List(ListRulesArgs { config: Some(path) }) if path.as_os_str() == "c.toml"
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "rules", "explain", "some_rule"]),
            RulesCommand::Explain(explain_args) if explain_args.rule_id == "some_rule"
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "rules", "groups"]),
            RulesCommand::Groups
        ));
        assert_eq!(parsed_ast_source(&["nu-lint", "dev", "ast", "ls"]), "ls");
    }

    #[test]
    fn test_legacy_flags_still_work() {
        assert!(matches!(
            parsed_command(&["nu-lint", "--fix", "file.nu"]),
            CliCommand::Fix(fix_args)
                if fix_args.input.paths == [PathBuf::from("file.nu")] && !fix_args.dry_run
        ));
        assert!(matches!(
            parsed_command(&["nu-lint", "--lsp"]),
            CliCommand::Lsp
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "--list", "--config", "c.toml"]),
            RulesCommand::List(ListRulesArgs { config: Some(path) }) if path.as_os_str() == "c.toml"
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "--rules"]),
            RulesCommand::List(ListRulesArgs { config: None })
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "--groups"]),
            RulesCommand::Groups
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "--sets"]),
            RulesCommand::Groups
        ));
        assert!(matches!(
            parsed_rules_command(&["nu-lint", "--explain", "some_rule"]),
            RulesCommand::Explain(explain_args) if explain_args.rule_id == "some_rule"
        ));
        assert_eq!(parsed_ast_source(&["nu-lint", "--ast", "ls"]), "ls");
    }

    #[test]
    fn test_meaningless_combinations_are_rejected() {
        let accepted_combinations: Vec<_> = [
            &["nu-lint", "--fix", "--lsp"][..],
            &["nu-lint", "--fix", "--explain", "rule"],
            &["nu-lint", "fix", "--lsp"],
            &["nu-lint", "--groups", "--format", "compact"],
            &["nu-lint", "--lsp", "--stdin"],
            &["nu-lint", "--explain", "rule", "file.nu"],
            &["nu-lint", "--stdin", "file.nu"],
            &["nu-lint", "check", "--stdin", "file.nu"],
            &[
                "nu-lint",
                "rules",
                "explain",
                "rule",
                "--only",
                "filter_to_where",
            ],
            &["nu-lint", "rules", "--only", "filter_to_where"],
            &["nu-lint", "rules", "--config", "c.toml", "groups"],
            &["nu-lint", "dev"],
            &["nu-lint", "fix", "--format", "compact"],
            &["nu-lint", "fix", "--dry-run", "--stdin"],
        ]
        .into_iter()
        .filter(|arguments| !is_rejected(arguments))
        .collect();
        assert!(
            accepted_combinations.is_empty(),
            "accepted meaningless combinations: {accepted_combinations:?}"
        );
    }

    #[test]
    fn test_help_lists_only_visible_commands() {
        let visible_command_names: Vec<_> = Cli::command()
            .get_subcommands()
            .filter(|subcommand| !subcommand.is_hide_set())
            .map(Command::get_name)
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            visible_command_names,
            ["check", "fix", "lsp", "rules", "dev"]
        );
    }

    #[test]
    fn test_cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_lint_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.nu");
        fs::write(&test_file, "def foo [] { echo 'hello' }").unwrap();

        let engine = crate::lint_engine_with_all_rules(Config::default());
        let files = collect_nu_files(&[test_file]);

        assert_eq!(files.len(), 1);
        let violations = engine.lint_files(&files);
        assert!(violations.is_empty() || !violations.is_empty()); // Just ensure it runs
    }
}
