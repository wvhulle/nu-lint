use std::{io, process::Command};

use crate::command_runner::{required_variable, run_checked, run_streaming};

const TEA_LOGIN_NAME: &str = "codeberg";
const CODEBERG_USER: &str = "wvhulle";

pub fn authenticate_git_origin() -> io::Result<()> {
    let codeberg_token = required_variable("CODEBERG_TOKEN")?;
    let repository_slug = required_variable("CI_REPO")?;
    let authenticated_url =
        format!("https://{CODEBERG_USER}:{codeberg_token}@codeberg.org/{repository_slug}.git");
    run_checked("git", &["remote", "set-url", "origin", &authenticated_url])?;
    Ok(())
}

pub struct CodebergRepository {
    repository_slug: String,
}

impl CodebergRepository {
    pub fn log_in() -> io::Result<Self> {
        let forge_url = required_variable("CI_FORGE_URL")?;
        let codeberg_token = required_variable("CODEBERG_TOKEN")?;
        let repository_slug = required_variable("CI_REPO")?;
        run_checked(
            "tea",
            &[
                "login",
                "add",
                "--name",
                TEA_LOGIN_NAME,
                "--url",
                &forge_url,
                "--token",
                &codeberg_token,
            ],
        )?;
        Ok(Self { repository_slug })
    }

    pub fn has_open_pull_request(&self, branch_name: &str) -> io::Result<bool> {
        let open_pr_heads = run_checked(
            "tea",
            &[
                "pulls",
                "list",
                "--login",
                TEA_LOGIN_NAME,
                "--repo",
                &self.repository_slug,
                "--state",
                "open",
                "--fields",
                "head",
                "--output",
                "simple",
            ],
        )?;
        Ok(open_pr_heads.lines().any(|head| head.trim() == branch_name))
    }

    pub fn open_or_keep_pull_request(
        &self,
        branch_name: &str,
        title: &str,
        description: &str,
    ) -> io::Result<()> {
        if self.has_open_pull_request(branch_name)? {
            println!("Updated existing pull request from {branch_name}");
            return Ok(());
        }
        run_checked(
            "tea",
            &[
                "pulls",
                "create",
                "--login",
                TEA_LOGIN_NAME,
                "--repo",
                &self.repository_slug,
                "--head",
                branch_name,
                "--base",
                "main",
                "--title",
                title,
                "--description",
                description,
            ],
        )?;
        println!("Opened pull request from {branch_name}");
        Ok(())
    }

    pub fn attach_release_assets(
        &self,
        release_tag: &str,
        asset_paths: &[String],
    ) -> io::Result<()> {
        run_streaming(
            Command::new("tea")
                .args(["releases", "assets", "create", "--login", TEA_LOGIN_NAME])
                .args(["--repo", &self.repository_slug, release_tag])
                .args(asset_paths),
        )
    }
}
