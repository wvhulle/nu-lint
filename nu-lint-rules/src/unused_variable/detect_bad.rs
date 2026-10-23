use super::RULE;

#[test]
fn test_unused_let() {
    RULE.assert_detects("let x = 5");
}

#[test]
fn test_unused_mut() {
    RULE.assert_detects("mut x = 5");
}

#[test]
fn test_unused_in_function() {
    let code = r#"
def foo [] {
    let unused = 5
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_multiple_unused() {
    let code = r#"
let a = 1
let b = 2
print "done"
"#;
    RULE.assert_count(code, 2);
}
