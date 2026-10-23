use super::RULE;

#[test]
fn fix_piped_line_count() {
    RULE.assert_fixed_is("$text | ^wc -l", "$text | lines | length");
}

#[test]
fn fix_file_line_count() {
    RULE.assert_fixed_is(
        r#"^wc -l "my notes.txt""#,
        r#"open --raw "my notes.txt" | lines | length"#,
    );
}
