#![cfg(feature = "lsp")]

use std::{
    io::Write,
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

const EXIT_DEADLINE: Duration = Duration::from_secs(10);

fn framed(json_body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{json_body}", json_body.len())
}

fn shutdown_conversation() -> String {
    [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#,
        r#"{"jsonrpc":"2.0","method":"exit","params":null}"#,
    ]
    .map(framed)
    .concat()
}

fn wait_until_exited(server: &mut Child) -> Option<ExitStatus> {
    let started_waiting = Instant::now();
    while started_waiting.elapsed() < EXIT_DEADLINE {
        if let Some(status) = server.try_wait().unwrap() {
            return Some(status);
        }
        thread::sleep(Duration::from_millis(50));
    }
    None
}

fn assert_exits_after_exit_notification(lsp_arguments: &[&str]) {
    let isolated_config_home = tempfile::tempdir().unwrap();
    let mut server = Command::new(env!("CARGO_BIN_EXE_nu-lint"))
        .args(lsp_arguments)
        .env("XDG_CONFIG_HOME", isolated_config_home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut client_input = server.stdin.take().unwrap();
    client_input
        .write_all(shutdown_conversation().as_bytes())
        .unwrap();
    client_input.flush().unwrap();

    let exit_status = wait_until_exited(&mut server);
    drop(client_input);
    if exit_status.is_none() {
        server.kill().unwrap();
    }

    let exit_status = exit_status.unwrap_or_else(|| {
        panic!(
            "`nu-lint {}` kept running {EXIT_DEADLINE:?} after the exit notification",
            lsp_arguments.join(" ")
        )
    });
    assert!(
        exit_status.success(),
        "unexpected exit status {exit_status:?}"
    );
}

fn assert_exits_when_client_disappears(lsp_arguments: &[&str]) {
    let isolated_config_home = tempfile::tempdir().unwrap();
    let mut server = Command::new(env!("CARGO_BIN_EXE_nu-lint"))
        .args(lsp_arguments)
        .env("XDG_CONFIG_HOME", isolated_config_home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut client_input = server.stdin.take().unwrap();
    client_input
        .write_all(
            framed(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#)
                .as_bytes(),
        )
        .unwrap();
    client_input
        .write_all(framed(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#).as_bytes())
        .unwrap();
    drop(client_input);

    let exit_status = wait_until_exited(&mut server);
    if exit_status.is_none() {
        server.kill().unwrap();
    }
    assert!(
        exit_status.is_some(),
        "`nu-lint {}` kept running {EXIT_DEADLINE:?} after its client closed stdin",
        lsp_arguments.join(" ")
    );
}

#[test]
fn lsp_exits_when_client_closes_stdin_without_shutdown() {
    assert_exits_when_client_disappears(&["--lsp"]);
}

#[test]
fn lsp_subcommand_exits_after_exit_notification() {
    assert_exits_after_exit_notification(&["lsp"]);
}

#[test]
fn legacy_lsp_flag_exits_after_exit_notification() {
    assert_exits_after_exit_notification(&["--lsp"]);
}
