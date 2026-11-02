use std::{
    io::Write,
    process::{Command, Stdio},
};

const PASTE_REPAIR_SELECTION: &str = "normalize_pasted_whitespace,translate-bash-syntax";

fn repair_pasted_text(pasted_text: &str) -> String {
    let empty_config = tempfile::NamedTempFile::new().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_nu-lint"))
        .args([
            "fix",
            "--stdin",
            "--only",
            PASTE_REPAIR_SELECTION,
            "--config",
        ])
        .arg(empty_config.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(pasted_text.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_string()
}

#[test]
fn repairs_chained_operators_in_one_pass() {
    assert_eq!(
        repair_pasted_text("^make && ^make test || ^echo failed\n"),
        "try { ^make; ^make test } catch { ^echo failed }",
    );
}

#[test]
fn repairs_backslash_line_continuation() {
    assert_eq!(
        repair_pasted_text("^docker run \\\n    --rm \\\n    alpine\n"),
        "^docker run --rm alpine",
    );
}

#[test]
fn repairs_tab_and_non_breaking_space() {
    assert_eq!(
        repair_pasted_text("^echo\tfirst\u{a0}second\n"),
        "^echo first second",
    );
}

#[test]
fn repairs_variable_next_to_operator() {
    assert_eq!(
        repair_pasted_text("^ls $HOME/projects && ^echo $?\n"),
        "^ls $\"($env.HOME)/projects\"; ^echo $env.LAST_EXIT_CODE",
    );
}
