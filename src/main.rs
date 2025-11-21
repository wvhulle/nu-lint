use std::{
    env::current_dir,
    io::{self, IsTerminal, Read},
    process,
};

use clap::{Parser, error::ErrorKind};
use nu_lint::{
    LintEngine,
    cli::{Cli, collect_files_to_lint, handle_command, lint_files, lint_stdin, output_results},
    config::Config,
    fix::{apply_fixes, format_fix_results},
};

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
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    let config = Config::load(cli.config.as_ref());

    if let Some(command) = cli.command {
        handle_command(command, &config);
        return;
    }

    let engine = LintEngine::new(config);

    let (all_violations, has_errors) = if cli.paths.is_empty() && !io::stdin().is_terminal() {
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
        lint_files(&engine, &files_to_lint, cli.parallel)
    };

    if has_errors && all_violations.is_empty() {
        process::exit(2);
    }

    // Apply fixes if requested
    if cli.fix || cli.dry_run {
        match apply_fixes(&all_violations, cli.dry_run) {
            Ok(results) => {
                println!("{}", format_fix_results(&results, cli.dry_run));
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

    output_results(&all_violations, cli.format);

    let exit_code = i32::from(!all_violations.is_empty());
    process::exit(exit_code);
}
