use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use crate::run_checked;

const RECORDINGS_BOOKMARK: &str = "recordings";
const RECORDINGS_COMMIT_MESSAGE: &str = "docs: regenerate recordings [skip ci]";
const STAGING_DIRECTORY_NAME: &str = "nu-lint-recordings";

pub fn record_scene(tape: &Path, output_directory: &Path) -> io::Result<()> {
    let scene_name = tape
        .file_stem()
        .ok_or_else(|| io::Error::other(format!("{} has no file name", tape.display())))?
        .to_string_lossy();
    let video_name = format!("{scene_name}.mp4");
    let webp_name = format!("{scene_name}.webp");
    run_checked("vhs", &[&tape.to_string_lossy()])?;
    run_checked(
        "ffmpeg",
        &[
            "-loglevel",
            "error",
            "-i",
            &video_name,
            "-vf",
            "mpdecimate,tpad=stop_mode=clone:stop_duration=5",
            "-fps_mode",
            "vfr",
            "-c:v",
            "libwebp_anim",
            "-lossless",
            "0",
            "-q:v",
            "80",
            "-compression_level",
            "6",
            "-loop",
            "0",
            &webp_name,
        ],
    )?;
    fs::create_dir_all(output_directory)?;
    fs::copy(&webp_name, output_directory.join(&webp_name))?;
    Ok(())
}

fn recorded_scene_paths(recordings: &Path) -> io::Result<Vec<PathBuf>> {
    fs::read_dir(recordings)?
        .map(|directory_entry| directory_entry.map(|entry| entry.path()))
        .filter(|scene_path| {
            scene_path.as_ref().map_or(true, |path| {
                path.extension()
                    .is_some_and(|extension| extension == "webp")
            })
        })
        .collect()
}

fn stage_recordings(scene_paths: &[PathBuf], staging_directory: &Path) -> io::Result<()> {
    if staging_directory.exists() {
        fs::remove_dir_all(staging_directory)?;
    }
    fs::create_dir_all(staging_directory)?;
    scene_paths.iter().try_for_each(|scene_path| {
        let scene_file_name = scene_path.file_name().ok_or_else(|| {
            io::Error::other(format!("{} has no file name", scene_path.display()))
        })?;
        fs::write(
            staging_directory.join(scene_file_name),
            fs::read(scene_path)?,
        )
    })
}

fn run_jj(staging_directory: &Path, arguments: &[&str]) -> io::Result<String> {
    let staging_path = staging_directory.to_string_lossy();
    let repository_arguments = [&["--repository", staging_path.as_ref()], arguments].concat();
    run_checked("jj", &repository_arguments)
}

pub fn publish_recordings(recordings: &Path, remote_url: &str) -> io::Result<()> {
    let scene_paths = recorded_scene_paths(recordings)?;
    if scene_paths.is_empty() {
        return Err(io::Error::other(format!(
            "no recordings in {}",
            recordings.display()
        )));
    }
    let staging_directory = env::temp_dir().join(STAGING_DIRECTORY_NAME);
    stage_recordings(&scene_paths, &staging_directory)?;

    run_checked("jj", &["git", "init", &staging_directory.to_string_lossy()])?;
    run_jj(
        &staging_directory,
        &["git", "remote", "add", "origin", remote_url],
    )?;
    run_jj(
        &staging_directory,
        &[
            "git",
            "fetch",
            "--remote",
            "origin",
            "--branch",
            RECORDINGS_BOOKMARK,
        ],
    )?;
    run_jj(
        &staging_directory,
        &["describe", "--message", RECORDINGS_COMMIT_MESSAGE],
    )?;

    let remote_bookmark = format!("{RECORDINGS_BOOKMARK}@origin");
    let listed_bookmarks = run_jj(
        &staging_directory,
        &[
            "bookmark",
            "list",
            "--all-remotes",
            "--template",
            "name ++ \"@\" ++ remote ++ \"\\n\"",
        ],
    )?;
    if listed_bookmarks
        .lines()
        .any(|listed_bookmark| listed_bookmark == remote_bookmark)
    {
        let changed_files = run_jj(
            &staging_directory,
            &["diff", "--summary", "--from", &remote_bookmark, "--to", "@"],
        )?;
        if changed_files.trim().is_empty() {
            println!("Recordings are unchanged, nothing to publish");
            return Ok(());
        }
        run_jj(
            &staging_directory,
            &["bookmark", "track", RECORDINGS_BOOKMARK, "--remote=origin"],
        )?;
        run_jj(
            &staging_directory,
            &[
                "bookmark",
                "set",
                RECORDINGS_BOOKMARK,
                "--revision",
                "@",
                "--allow-backwards",
            ],
        )?;
    } else {
        run_jj(
            &staging_directory,
            &["bookmark", "create", RECORDINGS_BOOKMARK, "--revision", "@"],
        )?;
        run_jj(
            &staging_directory,
            &["bookmark", "track", RECORDINGS_BOOKMARK, "--remote=origin"],
        )?;
    }
    run_jj(
        &staging_directory,
        &["git", "push", "--bookmark", RECORDINGS_BOOKMARK],
    )?;
    println!("Published {} recordings", scene_paths.len());
    Ok(())
}
