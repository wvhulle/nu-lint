mod codeberg_pull_request;
mod command_runner;
mod dependency_updates;
mod exported_nix_paths;
mod recordings;
mod release_tasks;

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use codeberg_pull_request::{CodebergRepository, authenticate_git_origin};
use command_runner::run_checked;

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
    #[command(about = "Nix post-build hook that exports built paths to the CI workspace")]
    ExportBuiltPathsHook,
    #[command(about = "Build flake outputs and export newly built paths to the CI workspace")]
    BuildAndExportPaths {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
        build_arguments: Vec<String>,
    },
    #[command(about = "Build and export a flake output unless it is already in Cachix")]
    BuildUnlessCached { installable: String },
    #[command(about = "Import paths exported to the CI workspace by an earlier step")]
    ImportExportedPaths,
    #[command(about = "Push paths exported to the CI workspace to Cachix")]
    UploadExportedPaths,
    #[command(about = "Publish an unreleased version to crates.io and tag a Codeberg release")]
    Release,
    #[command(about = "Open or update the release pull request")]
    ReleasePr,
    #[command(about = "Copy the archives of a build result into a release directory")]
    CollectReleaseArchives {
        build_result: PathBuf,
        dist_directory: PathBuf,
    },
    #[command(about = "Checksum a release directory and attach it to the Codeberg release")]
    UploadReleaseAssets { dist_directory: PathBuf },
    #[command(about = "Check that commit messages in revision ranges are conventional")]
    CheckCommits {
        #[arg(long, help = "First fetch the target branch of the pull request")]
        fetch_target_branch: bool,
        #[arg(required = true)]
        revision_ranges: Vec<String>,
    },
}

fn configuration_doc_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/configuration.md")
}

fn printed_example_config() -> io::Result<String> {
    run_checked(
        "cargo",
        &[
            "run",
            "--quiet",
            "--package",
            "nu-lint",
            "--",
            "config",
            "example",
        ],
    )
}

fn with_regenerated_example(configuration_doc: &str, example_config: &str) -> io::Result<String> {
    let missing_marker = || io::Error::other("example config markers not found");
    let (before_example, rest) = configuration_doc
        .split_once(START_MARKER)
        .ok_or_else(missing_marker)?;
    let (_, after_example) = rest.split_once(END_MARKER).ok_or_else(missing_marker)?;
    Ok(format!(
        "{before_example}{START_MARKER}```toml\n{example_config}```\n{END_MARKER}{after_example}"
    ))
}

fn open_example_config_pr(doc_path: &Path) -> io::Result<()> {
    let doc_path = doc_path.to_string_lossy();
    authenticate_git_origin()?;
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
    let regenerated_doc = with_regenerated_example(&configuration_doc, &printed_example_config()?)?;
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
        XtaskCommand::ExportBuiltPathsHook => exported_nix_paths::export_built_paths_hook(),
        XtaskCommand::BuildAndExportPaths { build_arguments } => {
            exported_nix_paths::build_and_export_paths(&build_arguments)
        }
        XtaskCommand::BuildUnlessCached { installable } => {
            exported_nix_paths::build_unless_cached(&installable)
        }
        XtaskCommand::ImportExportedPaths => exported_nix_paths::import_exported_paths(),
        XtaskCommand::UploadExportedPaths => exported_nix_paths::upload_exported_paths(),
        XtaskCommand::Release => release_tasks::publish_release(),
        XtaskCommand::ReleasePr => release_tasks::update_release_pr(),
        XtaskCommand::CollectReleaseArchives {
            build_result,
            dist_directory,
        } => release_tasks::collect_release_archives(&build_result, &dist_directory),
        XtaskCommand::UploadReleaseAssets { dist_directory } => {
            release_tasks::upload_release_assets(&dist_directory)
        }
        XtaskCommand::CheckCommits {
            fetch_target_branch,
            revision_ranges,
        } => release_tasks::check_commits(fetch_target_branch, &revision_ranges),
    };
    match task_outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(task_error) => {
            eprintln!("Error: {task_error}");
            ExitCode::FAILURE
        }
    }
}
