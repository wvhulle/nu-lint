use super::rule;

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
    rule().assert_ignores(good_code);
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
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_nu_without_c_flag() {
    let good_code = r#"nu script.nu"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_other_external_commands() {
    let good_code = r#"bash -c "echo hello""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_nu_help() {
    let good_code = r#"nu --help"#;
    rule().assert_ignores(good_code);
}
