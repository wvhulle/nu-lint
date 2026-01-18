use super::RULE;

#[test]
fn test_fix_equal_null_string_default() {
    let bad_code = r#"
def test [x] {
    if $x == null { "default" } else { $x }
}
"#;
    let expected = r#"
def test [x] {
    $x | default "default"
}
"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_not_equal_null() {
    let bad_code = r#"
def test [x] {
    if $x != null { $x } else { "fallback" }
}
"#;
    let expected = r#"
def test [x] {
    $x | default "fallback"
}
"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_null_on_left() {
    let bad_code = r#"
def test [value] {
    if null == $value { 0 } else { $value }
}
"#;
    let expected = r#"
def test [value] {
    $value | default 0
}
"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_numeric_default() {
    let bad_code = r#"
def test [count] {
    if $count == null { 0 } else { $count }
}
"#;
    let expected = r#"
def test [count] {
    $count | default 0
}
"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_list_default() {
    let bad_code = r#"
def test [items] {
    if $items == null { [] } else { $items }
}
"#;
    let expected = r#"
def test [items] {
    $items | default []
}
"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_record_default() {
    let bad_code = r#"
def test [config] {
    if $config == null { {} } else { $config }
}
"#;
    let expected = r#"
def test [config] {
    $config | default {}
}
"#;
    RULE.assert_fixed_is(bad_code, expected);
}
