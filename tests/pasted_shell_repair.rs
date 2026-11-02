use std::{
    io::Write,
    process::{Command, Stdio},
};

const PASTE_REPAIR_SELECTION: &str =
    "join_wrapped_lines,normalize_pasted_whitespace,translate-bash-syntax";

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
fn repairs_command_copied_from_wrapped_terminal_output() {
    let pasted_text = concat!(
        "    printf '%s\\n' '@nix {\"action\":\"result\",\"id\":4503307569594372,",
        "\"payload\":{\"bodySize\":15869172,\"httpStatus\":20\n",
        "      0},\"sid\":\"01a19be8-e20e-731d-803f-6fee2c300e2\",\"type\":10111}' ",
        "| <out>/bin/nom --json 2>&1 | grep -a -c\n",
        "      'nix-output-monitor error'\n",
    );

    assert_eq!(
        repair_pasted_text(pasted_text),
        concat!(
            "printf '%s\\n' '@nix {\"action\":\"result\",\"id\":4503307569594372,",
            "\"payload\":{\"bodySize\":15869172,\"httpStatus\":200},",
            "\"sid\":\"01a19be8-e20e-731d-803f-6fee2c300e2\",\"type\":10111}' ",
            "| <out>/bin/nom --json out+err>| grep -a -c 'nix-output-monitor error'",
        ),
    );
}

#[test]
fn repairs_chained_operators_in_one_pass() {
    assert_eq!(
        repair_pasted_text("^make && ^make test || ^echo failed\n"),
        "try { ^make; ^make test } catch { ^echo failed }",
    );
}

#[test]
fn repairs_line_continuation_without_overlapping_wrapped_line_join() {
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
