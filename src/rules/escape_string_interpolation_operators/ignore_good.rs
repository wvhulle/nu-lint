#[test]
fn ignores_escaped_parentheses() {
    let rule = super::rule();
    
    let code = r#"
def good_example [] {
    let path = "/some/path"
    print $"File ($path) \(not found\)"
}
"#;
    
    rule.assert_ignores(code);
}

#[test]
fn ignores_plain_strings() {
    let rule = super::rule();
    
    let code = r#"
def ok_example [] {
    print "(not a problem)"
}
"#;
    
    rule.assert_ignores(code);
}

#[test]
fn ignores_non_operator_keywords() {
    let rule = super::rule();
    
    let code = r#"
def ok_example [] {
    let x = "value"
    print $"Result ($x) (some text here)"
}
"#;
    
    rule.assert_ignores(code);
}

#[test]
fn ignores_escaped_all_keywords() {
    let rule = super::rule();
    
    let code = r#"
def good_operators [] {
    let var = "x"
    print $"($var) \(not x\)"
    print $"($var) \(and y\)"
    print $"($var) \(or z\)"
}
"#;
    
    rule.assert_ignores(code);
}

#[test]
fn ignores_simple_interpolation() {
    let rule = super::rule();
    
    let code = r#"
def test [] {
    let name = "Alice"
    print $"Hello ($name)"
}
"#;
    
    rule.assert_ignores(code);
}

#[test]
fn ignores_escaped_in_error_messages() {
    let rule = super::rule();

    let code = r#"
def good_error_msg [path: string] {
    error make {
        msg: $"File ($path) \(not found or not accessible\)"
    }
}
"#;

    rule.assert_ignores(code);
}

#[test]
fn ignores_valid_complex_expressions() {
    let rule = super::rule();

    let code = r#"
def test_valid_complex [] {
    let x = 5
    let y = 10
    print $"Check: (($x + $y) * 2)"
    print $"Result: (($x > 0) and ($y < 20))"
}
"#;

    rule.assert_ignores(code);
}

#[test]
fn ignores_properly_escaped_patterns() {
    let rule = super::rule();

    let code = r#"
def test_escaped [] {
    let file = "config.txt"
    print $"File ($file) \(not found\)"
    print $"Status ($file) \(access denied\)"
    print $"Service \(failed to connect\)"
    print $"Result ($file) \(and more\)"
}
"#;

    rule.assert_ignores(code);
}

#[test]
fn ignores_valid_subexpressions() {
    let rule = super::rule();

    let code = r#"
def test_valid [] {
    let a = 1
    let b = 2
    print $"Sum: (($a + $b))"
    print $"Product: (($a * $b * 3))"
}
"#;

    rule.assert_ignores(code);
}

#[test]
fn ignores_non_problematic_text() {
    let rule = super::rule();

    let code = r#"
def test_safe [] {
    let item = "data"
    print $"Item ($item) (version 1.0)"
    print $"Status ($item) (processing complete)"
    print $"Result ($item) (success)"
}
"#;

    rule.assert_ignores(code);
}