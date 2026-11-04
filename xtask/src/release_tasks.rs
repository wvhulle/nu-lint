use std::{fs, io, path::Path, process::Command};

use crate::{
    codeberg_pull_request::{CodebergRepository, authenticate_git_origin},
    command_runner::{required_variable, run_checked, run_streaming},
};

const RELEASE_ARCHIVE_SUFFIX: &str = ".tar.gz";
const CHECKSUM_FILE_NAME: &str = "sha256sum.txt";

fn run_release_plz(subcommand: &str) -> io::Result<()> {
    let codeberg_token = required_variable("CODEBERG_TOKEN")?;
    run_streaming(Command::new("release-plz").args([
        subcommand,
        "--forge",
        "gitea",
        "--git-token",
        &codeberg_token,
    ]))
}

pub fn publish_release() -> io::Result<()> {
    let commit_sha = required_variable("CI_COMMIT_SHA")?;
    authenticate_git_origin()?;
    run_checked("git", &["checkout", "-B", "main", &commit_sha])?;
    run_checked("git", &["fetch", "origin", "main:refs/remotes/origin/main"])?;
    run_checked("git", &["branch", "--set-upstream-to=origin/main", "main"])?;
    run_release_plz("release")
}

pub fn update_release_pr() -> io::Result<()> {
    run_release_plz("release-pr")
}

pub fn collect_release_archives(build_result: &Path, dist_directory: &Path) -> io::Result<()> {
    fs::create_dir_all(dist_directory)?;
    fs::read_dir(build_result)?.try_for_each(|directory_entry| {
        let archive_path = directory_entry?.path();
        let archive_name = archive_path.file_name().ok_or_else(|| {
            io::Error::other(format!("{} has no file name", archive_path.display()))
        })?;
        fs::copy(&archive_path, dist_directory.join(archive_name)).map(|_| ())
    })
}

fn is_release_archive(file_name: &str) -> bool {
    file_name.ends_with(RELEASE_ARCHIVE_SUFFIX)
}

fn sorted_file_names(dist_directory: &Path) -> io::Result<Vec<String>> {
    let mut file_names = fs::read_dir(dist_directory)?
        .map(|directory_entry| {
            directory_entry.map(|entry| entry.file_name().to_string_lossy().into_owned())
        })
        .collect::<io::Result<Vec<_>>>()?;
    file_names.sort();
    Ok(file_names)
}

fn write_checksum_file(dist_directory: &Path) -> io::Result<()> {
    let archive_names: Vec<String> = sorted_file_names(dist_directory)?
        .into_iter()
        .filter(|file_name| is_release_archive(file_name))
        .collect();
    let checksum_output = Command::new("sha256sum")
        .arg("--")
        .args(&archive_names)
        .current_dir(dist_directory)
        .output()?;
    if !checksum_output.status.success() {
        return Err(io::Error::other(format!(
            "sha256sum failed: {}",
            String::from_utf8_lossy(&checksum_output.stderr)
        )));
    }
    fs::write(
        dist_directory.join(CHECKSUM_FILE_NAME),
        checksum_output.stdout,
    )
}

pub fn upload_release_assets(dist_directory: &Path) -> io::Result<()> {
    let release_tag = required_variable("CI_COMMIT_TAG")?;
    write_checksum_file(dist_directory)?;
    let asset_paths: Vec<String> = sorted_file_names(dist_directory)?
        .iter()
        .map(|file_name| dist_directory.join(file_name).display().to_string())
        .collect();
    CodebergRepository::log_in()?.attach_release_assets(&release_tag, &asset_paths)
}

pub fn check_commits(fetch_target_branch: bool, revision_ranges: &[String]) -> io::Result<()> {
    if fetch_target_branch {
        let target_branch = required_variable("CI_COMMIT_TARGET_BRANCH")?;
        run_checked("git", &["fetch", "--quiet", "origin", &target_branch])?;
    }
    run_streaming(Command::new("convco").arg("check").args(revision_ranges))
}

#[cfg(test)]
mod tests {
    use super::is_release_archive;

    #[test]
    fn only_tarballs_are_checksummed() {
        assert!(
            is_release_archive("nu-lint-x86_64-unknown-linux-musl.tar.gz"),
            "release tarballs need a checksum"
        );
        assert!(
            !is_release_archive(super::CHECKSUM_FILE_NAME),
            "the checksum file must not list itself"
        );
    }
}
