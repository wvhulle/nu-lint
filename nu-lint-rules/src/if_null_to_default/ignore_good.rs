use super::RULE;

#[test]
fn test_ignore_already_using_default() {
    let good_code = r#"
def test [x] {
    $x | default "fallback"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_modified_variable_in_else() {
    // else block modifies the variable, not just returns it
    let good_code = r#"
def test [x] {
    if $x == null { 0 } else { $x + 1 }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_different_variable_in_else() {
    // else block returns a different variable
    let good_code = r#"
def test [x, y, z] {
    if $x == null { $y } else { $z }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_no_else_branch() {
    let good_code = r#"
def test [x] {
    if $x == null { print "missing" }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_else_if_chain() {
    let good_code = r#"
def test [x] {
    if $x == null { 0 } else if $x > 10 { 10 } else { $x }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_non_null_comparison() {
    let good_code = r#"
def test [x] {
    if $x == 0 { "zero" } else { $x }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_multiple_statements_in_then() {
    let good_code = r#"
def test [x] {
    if $x == null { print "null"; 0 } else { $x }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_multiple_statements_in_else() {
    let good_code = r#"
def test [x] {
    if $x == null { 0 } else { print $x; $x }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_boolean_comparison() {
    let good_code = r#"
def test [flag] {
    if $flag == true { "yes" } else { $flag }
}
"#;
    RULE.assert_ignores(good_code);
}
