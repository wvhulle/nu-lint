use std::process;

use clap::Parser;
use nu_lint::{
    cli::{Cli, collect_files_to_lint, handle_command, lint_files, load_config, output_results},
    LintEngine,
};


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
