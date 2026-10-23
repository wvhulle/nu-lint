use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use ::ignore::WalkBuilder;

fn is_nushell_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext == "nu")
        || fs::File::open(path)
            .ok()
            .and_then(|file| {
                let mut reader = io::BufReader::new(file);
                let mut first_line = String::new();
                reader.read_line(&mut first_line).ok()?;
                first_line.starts_with("#!").then(|| {
                    first_line
                        .split_whitespace()
                        .any(|word| word.ends_with("/nu") || word == "nu")
                })
            })
            .unwrap_or(false)
}

#[must_use]
pub fn collect_nu_files_from_dir(dir: &Path) -> Vec<PathBuf> {
    WalkBuilder::new(dir)
        .standard_filters(true)
        .build()
        .filter_map(|result| match result {
            Ok(entry) => {
                let path = entry.path().to_path_buf();
                (path.is_file() && is_nushell_file(&path)).then_some(path)
            }
            Err(err) => {
                log::warn!("Error walking directory: {err}");
                None
            }
        })
        .collect()
}

#[must_use]
pub fn collect_nu_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .flat_map(|path| {
            if !path.exists() {
                log::warn!("Path not found: {}", path.display());
                return vec![];
            }

            if path.is_file() {
                if is_nushell_file(path) {
                    vec![path.clone()]
                } else {
                    vec![]
                }
            } else if path.is_dir() {
                collect_nu_files_from_dir(path)
            } else {
                vec![]
            }
        })
        .collect()
}
