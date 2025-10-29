use super::rule;
use crate::context::LintContext;

#[test]
fn test_detect_sequential_without_check() {
    let bad_code = r"
^git add .
^git commit -m 'message'
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sequential_on_same_line() {
    let bad_code = r"^command1; ^command2";
    rule().assert_detects(bad_code);
}

#[test]
fn test_does_not_suggest_bash_and_operator() {
    let bad_code = r"
^git add .
^git commit -m 'message'
";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));
    assert!(
        !violations.is_empty(),
        "Should detect sequential external commands"
    );

    for violation in &violations {
        if let Some(suggestion) = &violation.suggestion {
            assert!(
                !suggestion.contains("&&"),
                "Should not suggest bash && operator in Nushell code. Suggestion was: {suggestion}"
            );
        }
    }
}
