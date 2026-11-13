#![cfg(feature = "lsp")]

mod lsp_conversation;

use std::{
    fs,
    io::{Read, Write},
    process::{Command, Stdio},
};

use lsp_conversation::shutdown_conversation;

fn server_stderr_with_arguments(lsp_arguments: &[&str]) -> String {
    let isolated_config_home = tempfile::tempdir().unwrap();
    let isolated_cache_home = tempfile::tempdir().unwrap();
    let mut server = Command::new(env!("CARGO_BIN_EXE_nu-lint"))
        .args(lsp_arguments)
        .env("XDG_CONFIG_HOME", isolated_config_home.path())
        .env("XDG_CACHE_HOME", isolated_cache_home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut client_input = server.stdin.take().unwrap();
    client_input
        .write_all(shutdown_conversation().as_bytes())
        .unwrap();
    drop(client_input);

    let mut server_stderr = String::new();
    server
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut server_stderr)
        .unwrap();
    assert!(server.wait().unwrap().success());
    server_stderr
}

#[test]
fn info_level_logs_server_lifecycle_to_stderr() {
    let server_stderr = server_stderr_with_arguments(&["lsp", "--log-level", "info"]);
    assert!(
        server_stderr.contains("nu-lint LSP server initialized"),
        "missing lifecycle log in stderr: {server_stderr:?}"
    );
    assert!(
        !server_stderr.contains("\u{1b}["),
        "stderr logs must not contain ANSI escapes: {server_stderr:?}"
    );
}

#[test]
fn default_level_keeps_stderr_free_of_info_logs() {
    let server_stderr = server_stderr_with_arguments(&["lsp"]);
    assert!(
        !server_stderr.contains("nu-lint LSP server initialized"),
        "info log leaked into default stderr: {server_stderr:?}"
    );
}

#[test]
fn log_file_receives_logs_at_the_chosen_level() {
    let scratch_directory = tempfile::tempdir().unwrap();
    let log_file = scratch_directory.path().join("nested").join("lsp.log");
    server_stderr_with_arguments(&[
        "lsp",
        "--log-level",
        "info",
        "--log-file",
        log_file.to_str().unwrap(),
    ]);
    let logged_text = fs::read_to_string(&log_file).unwrap();
    assert!(
        logged_text.contains("nu-lint LSP server initialized"),
        "missing lifecycle log in {logged_text:?}"
    );
    assert!(
        !logged_text.contains("\u{1b}["),
        "log file must not contain ANSI escapes: {logged_text:?}"
    );
}

#[test]
fn log_file_respects_the_default_warn_level() {
    let scratch_directory = tempfile::tempdir().unwrap();
    let log_file = scratch_directory.path().join("lsp.log");
    server_stderr_with_arguments(&["lsp", "--log-file", log_file.to_str().unwrap()]);
    let logged_text = fs::read_to_string(&log_file).unwrap();
    assert!(
        !logged_text.contains("nu-lint LSP server initialized"),
        "info log leaked into a warn-level log file: {logged_text:?}"
    );
}
