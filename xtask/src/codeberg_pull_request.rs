use std::{env, io, process::Command};

const TEA_LOGIN_NAME: &str = "codeberg";

pub fn run_checked(program: &str, arguments: &[&str]) -> io::Result<String> {
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

pub struct CodebergRepository {
    repository_slug: String,
}

impl CodebergRepository {
    pub fn log_in() -> io::Result<Self> {
        let forge_url = env::var("CI_FORGE_URL").map_err(io::Error::other)?;
        let codeberg_token = env::var("CODEBERG_TOKEN").map_err(io::Error::other)?;
        let repository_slug = env::var("CI_REPO").map_err(io::Error::other)?;
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
}
