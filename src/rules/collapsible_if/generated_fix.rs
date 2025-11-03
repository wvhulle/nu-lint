use crate::context::LintContext;

use super::rule;

#[test]
fn test_fix_simple_nested_if() {
    let bad_code = r#"if $x { if $y { print "yes" } }"#;
    let expected = r#"if $x and $y { print "yes" }"#;

    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert_eq!(violations.len(), 1, "Expected exactly one violation");
    let violation = &violations[0];

    assert!(violation.fix.is_some(), "Expected a fix to be available");
    let fix = violation.fix.as_ref().unwrap();

    assert_eq!(fix.replacements.len(), 1, "Expected one replacement");
    let replacement = &fix.replacements[0];

    assert_eq!(
        replacement.new_text, expected,
        "Fix produces incorrect code"
    );
}

#[test]
fn test_fix_nested_with_complex_conditions() {
    let bad_code = r#"if $x > 10 { if $y < 20 { echo "range" } }"#;
    let expected = r#"if $x > 10 and $y < 20 { echo "range" }"#;

    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert_eq!(violations.len(), 1);
    let violation = &violations[0];

    let fix = violation.fix.as_ref().unwrap();
    let replacement = &fix.replacements[0];

    assert_eq!(replacement.new_text, expected);
}

#[test]
fn test_fix_multiline_nested_if() {
    let bad_code = r#"
if $x > 0 {
    if $y > 0 {
        print "both positive"
    }
}
"#;
    let expected_condition = "$x > 0 and $y > 0";

    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert_eq!(violations.len(), 1);
    let violation = &violations[0];

    let fix = violation.fix.as_ref().unwrap();
    let replacement = &fix.replacements[0];

    assert!(
        replacement.new_text.contains(expected_condition),
        "Fix should contain combined condition: {}",
        replacement.new_text
    );
}

#[test]
fn test_fix_with_parentheses() {
    let bad_code = r"if ($enabled) { if ($ready) { start } }";
    let expected = r"if ($enabled) and ($ready) { start }";

    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    assert_eq!(violations.len(), 1);
    let violation = &violations[0];

    let fix = violation.fix.as_ref().unwrap();
    let replacement = &fix.replacements[0];

    assert_eq!(replacement.new_text, expected);
}
