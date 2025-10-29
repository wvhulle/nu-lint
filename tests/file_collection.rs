mod common;

use std::fs;

use nu_lint::cli::{collect_files_to_lint, collect_nu_files};
use tempfile::TempDir;

#[test]
fn test_collect_files_to_lint_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.nu");
    fs::write(&file_path, "let x = 5\n").unwrap();

    let files = collect_files_to_lint(std::slice::from_ref(&file_path));
    assert_eq!(files, vec![file_path]);
}

#[test]
fn test_collect_files_to_lint_directory() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("test1.nu");
    let file2 = temp_dir.path().join("test2.nu");
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file3 = subdir.join("test3.nu");

    fs::write(&file1, "let x = 5\n").unwrap();
    fs::write(&file2, "let y = 10\n").unwrap();
    fs::write(&file3, "let z = 15\n").unwrap();

    let collected_files = collect_files_to_lint(&[temp_dir.path().to_path_buf()]);
    assert_eq!(collected_files.len(), 3);
    assert!(collected_files.contains(&file1));
    assert!(collected_files.contains(&file2));
    assert!(collected_files.contains(&file3));
}

#[test]
fn test_collect_nu_files() {
    let temp_dir = TempDir::new().unwrap();
    let nu_file = temp_dir.path().join("test.nu");
    let other_file = temp_dir.path().join("test.txt");
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let nu_file_in_subdir = subdir.join("nested.nu");

    fs::write(&nu_file, "let x = 5\n").unwrap();
    fs::write(&other_file, "not a nu file\n").unwrap();
    fs::write(&nu_file_in_subdir, "let y = 10\n").unwrap();

    let files = collect_nu_files(&temp_dir.path().to_path_buf());
    assert_eq!(files.len(), 2);
    assert!(files.contains(&nu_file));
    assert!(files.contains(&nu_file_in_subdir));
    assert!(!files.contains(&other_file));
}

#[test]
fn test_lint_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Verify the directory exists but has no .nu files
    assert!(temp_dir.path().exists());
    let nu_files = collect_nu_files(&temp_dir.path().to_path_buf());
    assert!(nu_files.is_empty());
}
