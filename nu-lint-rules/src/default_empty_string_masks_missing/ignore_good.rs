use super::RULE;

#[test]
fn test_ignore_non_empty_string_default() {
    let good_code = r#"
def test [x] {
    $x | default "fallback"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_numeric_default() {
    let good_code = r#"
def test [x] {
    $x | default 0
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_empty_list_default() {
    let good_code = r#"
def test [x] {
    $x | default []
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_empty_record_default() {
    let good_code = r#"
def test [x] {
    $x | default {}
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_null_default() {
    let good_code = r#"
def test [x] {
    $x | default null
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_unrelated_call() {
    let good_code = r#"
def test [x] {
    $x | into string
}
"#;
    RULE.assert_ignores(good_code);
}
