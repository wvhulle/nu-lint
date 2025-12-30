use super::RULE;

#[test]
fn test_fix_simple_string_in_function() {
    let code = r#"
        def validate [input] {
            print --stderr "Invalid input"
        }
    "#;
    RULE.assert_fixed_contains(code, "error make");
    RULE.assert_fixed_contains(code, "msg:");
    RULE.assert_fixed_contains(code, "Invalid input");
}

#[test]
fn test_fix_variable_message_in_function() {
    let code = r#"
        def check [file] {
            print --stderr $msg
        }
    "#;
    RULE.assert_fixed_contains(code, "error make");
    RULE.assert_fixed_contains(code, "$msg");
}

#[test]
fn test_fix_interpolated_string_in_function() {
    let code = r#"
        def process [file: string] {
            print --stderr $"File not found: ($file)"
        }
    "#;
    RULE.assert_fixed_contains(code, "error make");
    RULE.assert_fixed_contains(code, "$\"File not found:");
}

#[test]
fn test_fix_in_try_block() {
    let code = r#"
        try {
            print --stderr "Operation failed"
        }
    "#;
    RULE.assert_fixed_contains(code, "error make");
    RULE.assert_fixed_contains(code, "Operation failed");
}

#[test]
fn test_fix_explanation() {
    let code = r#"
        def helper [] {
            print --stderr "Error"
        }
    "#;
    RULE.assert_fix_explanation_contains(code, "error make");
}
