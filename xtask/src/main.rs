use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::{Parser, Subcommand};
use nu_lint::example_config_toml;

const START_MARKER: &str = "<!-- start-example-config -->\n";
const END_MARKER: &str = "<!-- end-example-config -->";
const BOT_BRANCH: &str = "bot/example-config";
const BOT_COMMIT_MESSAGE: &str = "docs: regenerate example config";

#[derive(Parser)]
struct XtaskCli {
    #[command(subcommand)]
    command: XtaskCommand,
}

#[derive(Subcommand)]
enum XtaskCommand {
    #[command(about = "Regenerate the example config in docs/configuration.md")]
    UpdateExampleConfig {
        #[arg(
            long,
            help = "Commit, push and open a Codeberg pull request when it changed"
        )]
        open_pr: bool,
    },
}

fn configuration_doc_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/configuration.md")
}

fn with_regenerated_example(configuration_doc: &str) -> io::Result<String> {
    let missing_marker = || io::Error::other("example config markers not found");
    let (before_example, rest) = configuration_doc
        .split_once(START_MARKER)
        .ok_or_else(missing_marker)?;
    let (_, after_example) = rest.split_once(END_MARKER).ok_or_else(missing_marker)?;
    Ok(format!(
        "{before_example}{START_MARKER}```toml\n{}```\n{END_MARKER}{after_example}",
        example_config_toml()
    ))
}

fn run_checked(program: &str, arguments: &[&str]) -> io::Result<String> {
    let output = Command::new(program).args(arguments).output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{program} {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn open_example_config_pr(doc_path: &Path) -> io::Result<()> {
    let doc_path = doc_path.to_string_lossy();
    let forge_url = env::var("CI_FORGE_URL").map_err(io::Error::other)?;
    let codeberg_token = env::var("CODEBERG_TOKEN").map_err(io::Error::other)?;
    let repository = env::var("CI_REPO").map_err(io::Error::other)?;

    run_checked("git", &["checkout", "-B", BOT_BRANCH])?;
    run_checked(
        "git",
        &["commit", "--message", BOT_COMMIT_MESSAGE, "--", &doc_path],
    )?;
    run_checked("git", &["push", "--force", "origin", BOT_BRANCH])?;

    run_checked(
        "tea",
        &[
            "login",
            "add",
            "--name",
            "codeberg",
            "--url",
            &forge_url,
            "--token",
            &codeberg_token,
        ],
    )?;
    let open_pr_heads = run_checked(
        "tea",
        &[
            "pulls",
            "list",
            "--login",
            "codeberg",
            "--repo",
            &repository,
            "--state",
            "open",
            "--fields",
            "head",
            "--output",
            "simple",
        ],
    )?;
    if open_pr_heads.lines().any(|head| head.trim() == BOT_BRANCH) {
        println!("Updated existing pull request from {BOT_BRANCH}");
        return Ok(());
    }
    run_checked(
        "tea",
        &[
            "pulls",
            "create",
            "--login",
            "codeberg",
            "--repo",
            &repository,
            "--head",
            BOT_BRANCH,
            "--base",
            "main",
            "--title",
            BOT_COMMIT_MESSAGE,
            "--description",
            "Regenerated with `cargo xtask update-example-config` after the configuration options \
             changed.",
        ],
    )?;
    println!("Opened pull request from {BOT_BRANCH}");
    Ok(())
}

fn update_example_config(open_pr: bool) -> io::Result<()> {
    let doc_path = configuration_doc_path();
    let configuration_doc = fs::read_to_string(&doc_path)?;
    let regenerated_doc = with_regenerated_example(&configuration_doc)?;
    if regenerated_doc == configuration_doc {
        println!("Example config in {} is up to date", doc_path.display());
        return Ok(());
    }
    fs::write(&doc_path, regenerated_doc)?;
    println!("Regenerated example config in {}", doc_path.display());
    if open_pr {
        open_example_config_pr(&doc_path)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let XtaskCli { command } = XtaskCli::parse();
    let task_outcome = match command {
        XtaskCommand::UpdateExampleConfig { open_pr } => update_example_config(open_pr),
    };
    match task_outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(task_error) => {
            eprintln!("Error: {task_error}");
            ExitCode::FAILURE
        }
    }
}
