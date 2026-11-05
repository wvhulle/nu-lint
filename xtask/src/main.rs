mod codeberg_pull_request;
mod dependency_updates;
mod recordings;

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use codeberg_pull_request::{CodebergRepository, run_checked};
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
    #[command(about = "Update the Nushell crates, Cargo.lock and flake.lock")]
    UpdateDependencies {
        #[arg(
            long,
            help = "Commit, push and open a Codeberg pull request when changed"
        )]
        open_pr: bool,
    },
    #[command(about = "Record a VHS tape from the working directory as an animated WebP")]
    RecordScene {
        tape: PathBuf,
        #[arg(long, help = "Directory that receives the WebP")]
        out: PathBuf,
    },
    #[command(about = "Replace the recordings branch of a remote with the given WebP files")]
    PublishRecordings {
        recordings: PathBuf,
        remote_url: String,
    },
    #[command(
        about = "Replace the recordings branch on Codeberg, pushing with CODEBERG_TOKEN to CI_REPO"
    )]
    PublishRecordingsToCodeberg { recordings: PathBuf },
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

fn open_example_config_pr(doc_path: &Path) -> io::Result<()> {
    let doc_path = doc_path.to_string_lossy();
    let codeberg_repository = CodebergRepository::log_in()?;

    run_checked("git", &["checkout", "-B", BOT_BRANCH])?;
    run_checked(
        "git",
        &["commit", "--message", BOT_COMMIT_MESSAGE, "--", &doc_path],
    )?;
    run_checked("git", &["push", "--force", "origin", BOT_BRANCH])?;

    codeberg_repository.open_or_keep_pull_request(
        BOT_BRANCH,
        BOT_COMMIT_MESSAGE,
        "Regenerated with `cargo xtask update-example-config` after the configuration options \
         changed.",
    )
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
        XtaskCommand::UpdateDependencies { open_pr } => {
            dependency_updates::update_dependencies(open_pr)
        }
        XtaskCommand::RecordScene { tape, out } => recordings::record_scene(&tape, &out),
        XtaskCommand::PublishRecordings {
            recordings,
            remote_url,
        } => recordings::publish_recordings(&recordings, &remote_url),
        XtaskCommand::PublishRecordingsToCodeberg { recordings } => {
            recordings::publish_recordings_to_codeberg(&recordings)
        }
    };
    match task_outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(task_error) => {
            eprintln!("Error: {task_error}");
            ExitCode::FAILURE
        }
    }
}
