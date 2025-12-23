use super::rule;

#[test]
fn test_detect_nu_c_with_use_self() {
    let bad_code = r#"
def main [...args] {
    let script_path = $env.CURRENT_FILE
    nu -c $'use "($script_path)" *; mycommand ($args | str join " ")'
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_nu_c_simple() {
    let bad_code = r#"nu -c 'print "hello"'"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_nu_commands_flag() {
    let bad_code = r#"nu --commands 'ls | length'"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_nu_c_with_variable() {
    let bad_code = r#"
let cmd = "print hello"
nu -c $cmd
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_nu_c_in_function() {
    let bad_code = r#"
def dispatch [cmd: string] {
    nu -c $cmd
}
"#;
    rule().assert_detects(bad_code);
}
