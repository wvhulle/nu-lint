use super::RULE;

#[test]
fn test_detect_double_quoted_empty_string() {
    let bad_code = r#"
def test [x] {
    $x | default ""
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_single_quoted_empty_string() {
    let bad_code = r#"
def test [x] {
    $x | default ''
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_optional_env_var() {
    let bad_code = r#"
def test [] {
    $env.FOO? | default ""
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_let_binding() {
    let bad_code = r#"
def test [x] {
    let y = ($x | default "")
    $y
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_dispatch_match_idiom() {
    let bad_code = r#"
def main [...args: string] {
    match ($args | first | default "") {
        "sub" => { print "sub" }
        _ => { print "usage" }
    }
}
"#;
    RULE.assert_detects(bad_code);
}
