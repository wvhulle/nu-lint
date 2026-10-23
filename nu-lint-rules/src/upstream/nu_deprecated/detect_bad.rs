use super::RULE;

#[test]
fn detect_call_to_user_deprecated_command() {
    RULE.assert_detects(
        r#"
@deprecated "use new-greet instead"
def old-greet [] { "hi" }
old-greet
"#,
    );
}

#[test]
fn detect_only_unmigrated_deprecation_next_to_migrated_one() {
    RULE.assert_count(
        r#"
@deprecated "use new-greet instead"
def old-greet [] { "hi" }
old-greet | str upcase
"#,
        1,
    );
}
