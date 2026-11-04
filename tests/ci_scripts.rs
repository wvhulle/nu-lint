use std::{path::Path, process::Command};

#[test]
fn ci_scripts_pass_the_linter() {
    let ci_scripts = Path::new(env!("CARGO_MANIFEST_DIR")).join("ci");
    let lint_output = Command::new(env!("CARGO_BIN_EXE_nu-lint"))
        .arg(&ci_scripts)
        .output()
        .expect("nu-lint should start");
    assert!(
        lint_output.status.success(),
        "nu-lint reported violations in ci/:\n{}",
        String::from_utf8_lossy(&lint_output.stdout)
    );
}

#[test]
fn deliberate_failure_for_the_cache_upload_probe() {
    assert_eq!(1, 2, "this probe must fail so the upload runs after a failed build");
}
