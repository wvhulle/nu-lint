#[test]
fn detects_not_keyword() {
    let rule = super::rule();
    
    let code = r#"
def bad_example [] {
    let path = "/some/path"
    print $"File ($path) (not found)"
}
"#;
    
    rule.assert_detects(code);
}

#[test]
fn detects_and_keyword() {
    let rule = super::rule();
    
    let code = r#"
def test [] {
    let var = "x"
    print $"($var) (and y)"
}
"#;
    
    rule.assert_detects(code);
}

#[test]
fn detects_or_keyword() {
    let rule = super::rule();
    
    let code = r#"
def test [] {
    let var = "x"
    print $"($var) (or z)"
}
"#;
    
    rule.assert_detects(code);
}

#[test]
fn detects_multiple_violations() {
    let rule = super::rule();
    
    let code = r#"
def test [] {
    let name = "test"
    print $"Item ($name) (or maybe not) (and another)"
}
"#;
    
    rule.assert_detects(code);
}

#[test]
fn detects_in_error_messages() {
    let rule = super::rule();
    
    let code = r#"
def bad_error_msg [path: string] {
    error make {
        msg: $"File ($path) (not found or not accessible)"
    }
}
"#;
    
    rule.assert_detects(code);
}

#[test]
fn detects_nested_expressions() {
    let rule = super::rule();

    let code = r#"
def bad_nested [] {
    let a = "x"
    let b = "y"
    print $"Values ($a) and ($b) (not equal)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_error_message_patterns() {
    let rule = super::rule();

    let code = r#"
def test_errors [] {
    let file = "config.txt"
    print $"File ($file) (not found)"
    print $"Access ($file) (access denied)"
    print $"Service (failed to connect)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_incomplete_expressions() {
    let rule = super::rule();

    let code = r#"
def test_incomplete [] {
    let x = "value"
    print $"Result ($x) (==)"
    print $"Status ($x) (+)"
    print $"Check ($x) (if condition)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_complex_boolean_logic() {
    let rule = super::rule();

    let code = r#"
def test_complex [] {
    let x = "test"
    print $"Check ($x) (x > 0 and y < 10)"
    print $"Validate ($x) (not empty and not null)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_common_error_patterns() {
    let rule = super::rule();

    let code = r#"
def test_common [] {
    let item = "data"
    print $"Item ($item) (not available)"
    print $"Status ($item) (and more)"
    print $"Result ($item) (or else)"
    print $"Service ($item) (unable to process)"
}
"#;

    rule.assert_detects(code);
}
