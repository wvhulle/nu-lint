use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

fn violating_workspace() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for index in 0..400 {
        let mut file = fs::File::create(dir.path().join(format!("script{index}.nu"))).unwrap();
        writeln!(file, "def main [] {{ echo 'hello' }}").unwrap();
    }
    dir
}

#[test]
fn closing_stdout_early_exits_successfully() {
    let workspace = violating_workspace();

    let mut child = Command::new(env!("CARGO_BIN_EXE_nu-lint"))
        .arg(workspace.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    drop(child.stdout.take().unwrap());

    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "expected a clean exit after the pipe closed, got {:?} with stderr: {stderr}",
        output.status
    );
    assert!(
        !stderr.contains("panicked"),
        "nu-lint panicked on a broken pipe: {stderr}"
    );
}
