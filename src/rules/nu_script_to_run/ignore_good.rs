use super::RULE;

#[test]
fn ignore_run_keyword() {
    RULE.assert_ignores("'hello' | run shout.nu");
}

#[test]
fn ignore_nu_commands_flag() {
    RULE.assert_ignores("^nu -c 'print hello'");
}

#[test]
fn ignore_dynamic_script_path() {
    let good_code = r#"
def main [script: path] {
    ^nu $script
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_interpreter_flags() {
    let good_code = r#"
^nu --no-config-file build.nu
^nu -n build.nu
^nu --config custom.nu build.nu
^nu --experimental-options '[pipefail]' build.nu
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_non_nu_file() {
    RULE.assert_ignores("^nu script.sh");
}

#[test]
fn ignore_other_interpreters() {
    RULE.assert_ignores("^python build.nu");
}

#[test]
fn ignore_nu_without_arguments() {
    RULE.assert_ignores("^nu");
}

#[test]
fn ignore_interpolated_script_path() {
    let good_code = r#"
let name = 'build'
^nu $"($name).nu"
"#;
    RULE.assert_ignores(good_code);
}
