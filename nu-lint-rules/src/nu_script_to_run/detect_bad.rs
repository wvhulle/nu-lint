use super::RULE;

#[test]
fn detect_nu_script_without_input() {
    RULE.assert_detects("^nu build.nu");
}

#[test]
fn detect_nu_script_with_arguments() {
    RULE.assert_detects("^nu deploy.nu --target staging 3");
}

#[test]
fn detect_piped_input_with_stdin_flag() {
    RULE.assert_detects("'hello' | ^nu --stdin shout.nu");
}

#[test]
fn detect_quoted_script_path() {
    RULE.assert_detects("^nu 'scripts/format output.nu'");
}

#[test]
fn detect_inside_function() {
    let bad_code = r#"
def main [] {
    ls | get name | to json | ^nu --stdin summarize.nu | lines
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_bare_nu_call() {
    RULE.assert_detects("nu tools/check.nu");
}
