use std::{
    env::current_dir,
    io::{self, IsTerminal, Read},
    process,
};

use clap::{Parser, error::ErrorKind};
use nu_lint::{
    LintEngine, Violation,
    cli::{Cli, collect_files_to_lint, handle_command, lint_files, lint_stdin, output_results},
    config::Config,
    fix::{apply_fixes, apply_fixes_to_stdin, format_fix_results},
    log::instrument,
};

fn handle_fixes(violations: &[Violation], is_stdin: bool, dry_run: bool) {
    if is_stdin {
        handle_stdin_fixes(violations, dry_run);
    } else {
        handle_file_fixes(violations, dry_run);
    }
}

fn handle_stdin_fixes(violations: &[Violation], dry_run: bool) {
    if let Some(fixed_content) = apply_fixes_to_stdin(violations) {
        if dry_run {
            eprintln!("Fixed content (dry-run):");
        }
        print!("{fixed_content}");
        process::exit(0);
    }
    eprintln!("No fixable violations found in stdin.");
    process::exit(0);
}

fn handle_file_fixes(violations: &[Violation], dry_run: bool) {
    match apply_fixes(violations, dry_run) {
        Ok(results) => {
            println!("{}", format_fix_results(&results, dry_run));
            if !results.is_empty() {
                process::exit(0);
            }
        }
        Err(e) => {
            eprintln!("Error applying fixes: {e}");
            process::exit(2);
        }
    }
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            ErrorKind::UnknownArgument => {
                eprintln!("Error: Unknown argument or option");
                eprintln!();
                eprintln!(
                    "Usage: {} [OPTIONS] [PATHS]... [COMMAND]",
                    env!("CARGO_PKG_NAME")
                );
                eprintln!();
                eprintln!("For more information, try '--help'");
                process::exit(2);
            }
            _ => {
                err.exit();
            }
        },
    };

    // Initialize logging based on verbose flag
    if cli.verbose {
        instrument();
    }

    let config = Config::load(cli.config.as_ref());

    if let Some(command) = cli.command {
        handle_command(command, &config);
        return;
    }

    let engine = LintEngine::new(config);

    let is_stdin = cli.paths.is_empty() && !io::stdin().is_terminal();

    let (all_violations, has_errors) = if is_stdin {
        let mut input = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut input) {
            eprintln!("Error reading from stdin: {e}");
            process::exit(2);
        }
        lint_stdin(&engine, &input)
    } else {
        let paths_to_lint = if cli.paths.is_empty() {
            vec![current_dir().unwrap_or_else(|_| {
                eprintln!("Error: Unable to determine current directory");
                process::exit(2);
            })]
        } else {
            cli.paths
        };

        let files_to_lint = collect_files_to_lint(&paths_to_lint);
        lint_files(&engine, &files_to_lint)
    };

    if has_errors && all_violations.is_empty() {
        process::exit(2);
    }

    // Apply fixes if requested
    if cli.fix || cli.dry_run {
        handle_fixes(&all_violations, is_stdin, cli.dry_run);
    }

    output_results(&all_violations, cli.format);

    let exit_code = i32::from(!all_violations.is_empty());
    process::exit(exit_code);
}
