use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use crate::command_runner::{required_variable, run_checked, run_streaming};

const EXPORT_DIRECTORY_NAME: &str = ".nix-built-paths";
const PATH_LIST_FILE_NAME: &str = "paths.txt";
const CACHIX_CACHE_NAME: &str = "wvhulle";
const CACHIX_CACHE_URL: &str = "https://wvhulle.cachix.org";
const NIX_STORE_DIRECTORY: &str = "/nix/store";
const HOOK_PROGRAM_VARIABLE: &str = "EXPORT_HOOK_PROGRAM";

fn export_directory() -> io::Result<PathBuf> {
    Ok(PathBuf::from(required_variable("CI_WORKSPACE")?).join(EXPORT_DIRECTORY_NAME))
}

fn binary_cache_url(export_directory: &Path) -> String {
    format!("file://{}", export_directory.display())
}

fn listed_store_paths(path_list: &str) -> Vec<&str> {
    path_list.lines().filter(|line| !line.is_empty()).collect()
}

fn read_path_list(export_directory: &Path) -> io::Result<String> {
    let path_list_file = export_directory.join(PATH_LIST_FILE_NAME);
    if path_list_file.exists() {
        fs::read_to_string(path_list_file)
    } else {
        Ok(String::new())
    }
}

pub fn export_built_paths_hook() -> io::Result<()> {
    let export_directory = export_directory()?;
    let built_paths = required_variable("OUT_PATHS")?;
    fs::create_dir_all(&export_directory)?;
    run_streaming(
        Command::new("nix")
            .args(["copy", "--quiet", "--to"])
            .arg(format!(
                "{}?compression=zstd",
                binary_cache_url(&export_directory)
            ))
            .args(built_paths.split_whitespace()),
    )?;
    let mut path_list_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(export_directory.join(PATH_LIST_FILE_NAME))?;
    built_paths
        .split_whitespace()
        .try_for_each(|built_path| writeln!(path_list_file, "{built_path}"))
}

fn nix_config_with_hook(existing_config: &str, hook_program: &str) -> String {
    format!("{existing_config}\npost-build-hook = {hook_program}")
}

pub fn build_and_export_paths(build_arguments: &[String]) -> io::Result<()> {
    let hook_program = required_variable(HOOK_PROGRAM_VARIABLE)?;
    let existing_config = env::var("NIX_CONFIG").unwrap_or_default();
    run_streaming(
        Command::new("nix")
            .args(["build", "--quiet", "--print-build-logs"])
            .args(build_arguments)
            .env(
                "NIX_CONFIG",
                nix_config_with_hook(&existing_config, &hook_program),
            ),
    )
}

pub fn build_unless_cached(installable: &str) -> io::Result<()> {
    let output_attribute = format!("{installable}.outPath");
    let built_output = run_checked("nix", &["eval", "--quiet", "--raw", &output_attribute])?;
    let is_cached = Command::new("nix")
        .args(["path-info", "--quiet", "--store", CACHIX_CACHE_URL])
        .arg(&built_output)
        .output()?
        .status
        .success();
    if is_cached {
        println!("{built_output} is already in Cachix");
        return Ok(());
    }
    build_and_export_paths(&[installable.to_owned()])
}

pub fn import_exported_paths() -> io::Result<()> {
    let export_directory = export_directory()?;
    let path_list = read_path_list(&export_directory)?;
    let exported_paths = listed_store_paths(&path_list);
    if exported_paths.is_empty() {
        println!("No exported paths to import");
        return Ok(());
    }
    run_streaming(
        Command::new("nix")
            .args(["copy", "--quiet", "--no-check-sigs", "--from"])
            .arg(binary_cache_url(&export_directory))
            .args(exported_paths),
    )
}

fn containing_store_path(program_path: &Path) -> Option<&Path> {
    program_path
        .ancestors()
        .find(|ancestor| ancestor.parent() == Some(Path::new(NIX_STORE_DIRECTORY)))
}

fn running_xtask_store_path() -> io::Result<Option<PathBuf>> {
    let program_path = fs::canonicalize(env::current_exe()?)?;
    Ok(containing_store_path(&program_path).map(Path::to_path_buf))
}

pub fn upload_exported_paths() -> io::Result<()> {
    import_exported_paths()?;
    let path_list = read_path_list(&export_directory()?)?;
    let exported_paths = listed_store_paths(&path_list);
    let xtask_store_path = running_xtask_store_path()?;
    if exported_paths.is_empty() && xtask_store_path.is_none() {
        println!("No newly built paths to upload");
        return Ok(());
    }
    run_streaming(
        Command::new("cachix")
            .args(["push", CACHIX_CACHE_NAME])
            .args(exported_paths)
            .args(xtask_store_path),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_appended_after_existing_config() {
        assert_eq!(
            nix_config_with_hook("max-jobs = 1", "/nix/store/hook"),
            "max-jobs = 1\npost-build-hook = /nix/store/hook",
            "the hook setting must be on its own line"
        );
    }

    #[test]
    fn empty_lines_are_not_store_paths() {
        assert_eq!(
            listed_store_paths("/nix/store/first\n\n/nix/store/second\n"),
            ["/nix/store/first", "/nix/store/second"],
            "only store paths may reach nix copy"
        );
    }

    #[test]
    fn program_inside_store_resolves_to_its_store_path() {
        assert_eq!(
            containing_store_path(Path::new("/nix/store/abc-xtask/bin/xtask")),
            Some(Path::new("/nix/store/abc-xtask")),
            "cachix needs the store path, not the program inside it"
        );
    }

    #[test]
    fn program_outside_store_has_no_store_path() {
        assert_eq!(
            containing_store_path(Path::new("/home/user/target/debug/xtask")),
            None,
            "a local build must not be pushed"
        );
    }
}
