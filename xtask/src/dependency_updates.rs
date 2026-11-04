use std::io;

use crate::{
    codeberg_pull_request::{CodebergRepository, authenticate_git_origin},
    command_runner::run_checked,
};

const UPDATE_BRANCH: &str = "bot/dependency-updates";
const UPDATE_BOT_AUTHOR: &str = "nu-lint-bot";
const MANIFEST_FILE_NAME: &str = "Cargo.toml";
const XTASK_MANIFEST_PATH: &str = "xtask/Cargo.toml";
const MANIFEST_COMMIT_MESSAGE: &str = "fix(deps): update nushell crates and lockfiles";
const LOCKFILE_COMMIT_MESSAGE: &str = "chore(deps): update lockfiles";
const PULL_REQUEST_DESCRIPTION: &str = "Updated with `cargo xtask update-dependencies --open-pr`. \
                                        Fixes pushed to this branch stop further automatic \
                                        updates until it is merged.";
const NUSHELL_CRATES: [&str; 7] = [
    "nu-cli",
    "nu-cmd-extra",
    "nu-cmd-lang",
    "nu-command",
    "nu-parser",
    "nu-protocol",
    "nu-std",
];

fn commit_message_for_changed_paths(changed_paths: &str) -> &'static str {
    if changed_paths
        .lines()
        .any(|changed_path| changed_path.ends_with(MANIFEST_FILE_NAME))
    {
        MANIFEST_COMMIT_MESSAGE
    } else {
        LOCKFILE_COMMIT_MESSAGE
    }
}

fn has_hand_written_commits(commit_authors: &str) -> bool {
    commit_authors
        .lines()
        .any(|commit_author| commit_author != UPDATE_BOT_AUTHOR)
}

fn upgrade_dependency_versions() -> io::Result<()> {
    let package_arguments = NUSHELL_CRATES
        .iter()
        .flat_map(|crate_name| ["--package", crate_name]);
    let upgrade_arguments: Vec<&str> = ["upgrade", "--incompatible", "allow"]
        .into_iter()
        .chain(package_arguments)
        .collect();
    run_checked("cargo", &upgrade_arguments)?;
    run_checked("cargo", &["update"])?;
    run_checked("cargo", &["update", "--manifest-path", XTASK_MANIFEST_PATH])?;
    run_checked("nix", &["flake", "update"])?;
    Ok(())
}

fn changed_tracked_paths() -> io::Result<String> {
    run_checked("git", &["diff", "--name-only"])
}

fn remote_update_branch_commit() -> io::Result<Option<String>> {
    let update_branch_ref = format!("refs/heads/{UPDATE_BRANCH}");
    let listed_refs = run_checked(
        "git",
        &["ls-remote", "--heads", "origin", &update_branch_ref],
    )?;
    Ok(listed_refs.split_whitespace().next().map(ToOwned::to_owned))
}

fn fetch_remote_branch(branch_name: &str) -> io::Result<()> {
    let tracking_refspec = format!("+refs/heads/{branch_name}:refs/remotes/origin/{branch_name}");
    run_checked("git", &["fetch", "origin", &tracking_refspec])?;
    Ok(())
}

fn update_branch_has_hand_written_commits() -> io::Result<bool> {
    fetch_remote_branch(UPDATE_BRANCH)?;
    let commit_range = format!("origin/main..origin/{UPDATE_BRANCH}");
    let commit_authors = run_checked("git", &["log", "--format=%an", &commit_range])?;
    Ok(has_hand_written_commits(&commit_authors))
}

fn open_dependency_update_pr() -> io::Result<()> {
    authenticate_git_origin()?;
    let codeberg_repository = CodebergRepository::log_in()?;
    fetch_remote_branch("main")?;
    let remote_commit = remote_update_branch_commit()?;

    if remote_commit.is_some()
        && codeberg_repository.has_open_pull_request(UPDATE_BRANCH)?
        && update_branch_has_hand_written_commits()?
    {
        println!("Leaving {UPDATE_BRANCH} alone because it contains hand-written commits");
        return Ok(());
    }

    run_checked("git", &["checkout", "-B", UPDATE_BRANCH, "origin/main"])?;
    upgrade_dependency_versions()?;

    let changed_paths = changed_tracked_paths()?;
    if changed_paths.is_empty() {
        println!("Dependencies are up to date");
        return Ok(());
    }

    let commit_message = commit_message_for_changed_paths(&changed_paths);
    run_checked("git", &["commit", "--all", "--message", commit_message])?;
    let force_with_lease = format!(
        "--force-with-lease={UPDATE_BRANCH}:{}",
        remote_commit.unwrap_or_default()
    );
    run_checked("git", &["push", &force_with_lease, "origin", UPDATE_BRANCH])?;
    codeberg_repository.open_or_keep_pull_request(
        UPDATE_BRANCH,
        commit_message,
        PULL_REQUEST_DESCRIPTION,
    )
}

pub fn update_dependencies(open_pr: bool) -> io::Result<()> {
    if open_pr {
        return open_dependency_update_pr();
    }
    upgrade_dependency_versions()?;
    print!("{}", changed_tracked_paths()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_change_uses_releasing_commit_message() {
        assert_eq!(
            commit_message_for_changed_paths("Cargo.lock\nCargo.toml\nflake.lock\n"),
            MANIFEST_COMMIT_MESSAGE,
            "a manifest change must trigger a release"
        );
    }

    #[test]
    fn lockfile_change_uses_non_releasing_commit_message() {
        assert_eq!(
            commit_message_for_changed_paths("Cargo.lock\nflake.lock\n"),
            LOCKFILE_COMMIT_MESSAGE,
            "a lockfile change must not trigger a release"
        );
    }

    #[test]
    fn bot_commits_are_not_hand_written() {
        assert!(
            !has_hand_written_commits("nu-lint-bot\nnu-lint-bot\n"),
            "commits by the bot may be replaced"
        );
    }

    #[test]
    fn other_author_counts_as_hand_written() {
        assert!(
            has_hand_written_commits("Willem Vanhulle\nnu-lint-bot\n"),
            "commits by anyone else must be kept"
        );
    }
}
