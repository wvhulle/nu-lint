use super::RULE;

#[test]
fn fix_piped_head_uses_default_count() {
    RULE.assert_fixed_is("$text | ^head", "$text | lines | first 10");
}

#[test]
fn fix_count_forms() {
    for code in [
        "$text | ^head -n 5",
        "$text | ^head -n5",
        "$text | ^head -5",
        "$text | ^head --lines=5",
    ] {
        RULE.assert_fixed_is(code, "$text | lines | first 5");
    }
}

#[test]
fn fix_piped_tail() {
    RULE.assert_fixed_is("$text | ^tail -n 3", "$text | lines | last 3");
}

#[test]
fn fix_tail_from_line() {
    RULE.assert_fixed_is("$text | ^tail -n +2", "$text | lines | skip 1");
}

#[test]
fn fix_variable_count() {
    RULE.assert_fixed_is(
        "let n = 4; $text | ^head -n $n",
        "let n = 4; $text | lines | first $n",
    );
}

#[test]
fn fix_single_file_keeps_quotes() {
    RULE.assert_fixed_is(
        r#"^tail -n 2 "my log.txt""#,
        r#"open --raw "my log.txt" | lines | last 2"#,
    );
}
