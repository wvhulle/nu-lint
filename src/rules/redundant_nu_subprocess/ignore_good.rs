use super::RULE;

#[test]
fn test_ignore_direct_function_call() {
    let good_code = r#"
def greet [name: string] {
    print $"Hello ($name)"
}

def main [] {
    greet "world"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_match_dispatch() {
    let good_code = r#"
def main [...args: string] {
    match ($args | first | default "") {
        "greet" => { print "hello" }
        _ => { print "usage" }
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_nu_without_c_flag() {
    let good_code = r#"nu script.nu"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_other_external_commands() {
    let good_code = r#"bash -c "echo hello""#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_nu_help() {
    let good_code = r#"nu --help"#;
    RULE.assert_ignores(good_code);
}
