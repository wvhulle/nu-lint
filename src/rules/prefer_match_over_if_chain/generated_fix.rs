use super::rule;
use crate::{LintContext, clean_log::log};
// These tests verify that the automatic fix correctly converts if-else-if
// chains into valid match expressions with proper variable names and syntax.

#[test]
fn test_simple_string_chain_fix() {
    log();
    let bad_code = r#"
if $status == "ok" {
    "success"
} else if $status == "error" {
    "failed"
} else if $status == "pending" {
    "waiting"
} else {
    "unknown"
}
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        let violation = &violations[0];

        let fix = violation.fix.as_ref().expect("Should have automatic fix");
        assert_eq!(fix.replacements.len(), 1, "Should have one replacement");

        let replacement = &fix.replacements[0];
        let new_text = replacement.new_text.as_ref();

        // Verify the match expression uses the correct variable
        assert!(
            new_text.contains("match $status"),
            "Should use correct variable name: {new_text}"
        );

        // Verify match arms are present
        assert!(
            new_text.contains(r#""ok" =>"#),
            "Should have match arm for ok: {new_text}"
        );
        assert!(
            new_text.contains(r#""error" =>"#),
            "Should have match arm for error: {new_text}"
        );
        assert!(
            new_text.contains(r#""pending" =>"#),
            "Should have match arm for pending: {new_text}"
        );

        // Verify catch-all pattern
        assert!(
            new_text.contains("_ =>"),
            "Should have catch-all pattern: {new_text}"
        );

        // Verify bodies are preserved
        assert!(
            new_text.contains(r#""success""#),
            "Should preserve success body: {new_text}"
        );
        assert!(
            new_text.contains(r#""failed""#),
            "Should preserve failed body: {new_text}"
        );
    });
}

#[test]
fn test_numeric_chain_fix() {
    let bad_code = r#"
if $code == 200 {
    "ok"
} else if $code == 404 {
    "not found"
} else if $code == 500 {
    "error"
} else {
    "unknown"
}
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        let violation = &violations[0];

        let fix = violation.fix.as_ref().expect("Should have automatic fix");
        let new_text = fix.replacements[0].new_text.as_ref();

        assert!(
            new_text.contains("match $code"),
            "Should use correct variable name: {new_text}"
        );
        assert!(
            new_text.contains("200 =>"),
            "Should have match arm for 200: {new_text}"
        );
        assert!(
            new_text.contains("404 =>"),
            "Should have match arm for 404: {new_text}"
        );
        assert!(
            new_text.contains("500 =>"),
            "Should have match arm for 500: {new_text}"
        );
        assert!(
            new_text.contains("_ =>"),
            "Should have catch-all for unknown codes: {new_text}"
        );
    });
}
