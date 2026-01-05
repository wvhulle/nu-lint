use super::RULE;

#[test]
fn fix_simple_grep_to_find() {
    let source = r#"^grep "pattern""#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"find "pattern""#);
}

#[test]
fn fix_grep_with_file_to_where() {
    let source = r#"^grep "error" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"open logs.txt | lines | where $it =~ "error""#);
}

#[test]
fn fix_grep_case_insensitive() {
    let source = r#"^grep -i "warning" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"open logs.txt | lines | where $it =~ "warning""#);
    RULE.assert_fix_explanation_contains(source, "redundant");
    RULE.assert_fix_explanation_contains(source, "-i");
}

#[test]
fn fix_grep_invert_match() {
    let source = r#"^grep -v "debug" app.log"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"open app.log | lines | where $it !~ "debug""#);
    RULE.assert_fix_explanation_contains(source, "!~");
}

#[test]
fn fix_grep_line_number() {
    let source = r#"^grep -n "TODO" source.rs"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open source.rs | lines | enumerate | where $it =~ "TODO""#,
    );
    RULE.assert_fix_explanation_contains(source, "enumerate");
}

#[test]
fn fix_grep_count() {
    let source = r#"^grep -c "error" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open logs.txt | lines | where $it =~ "error" | length"#,
    );
    RULE.assert_fix_explanation_contains(source, "length");
}

#[test]
fn fix_grep_fixed_strings() {
    let source = r#"^grep -F "literal.string" file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open file.txt | lines | where $it | str contains "literal.string""#,
    );
    RULE.assert_fix_explanation_contains(source, "str contains");
}

#[test]
fn fix_grep_recursive() {
    let source = r#"^grep -r "TODO" ."#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"open . | lines | where $it =~ "TODO""#);
}

#[test]
fn fix_ripgrep_simple() {
    let source = r#"^rg "pattern""#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"find "pattern""#);
}

#[test]
fn fix_ripgrep_with_file() {
    let source = r#"^rg "error" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"open logs.txt | lines | where $it =~ "error""#);
}

#[test]
fn fix_description_mentions_find() {
    let source = r#"^grep "pattern""#;
    RULE.assert_count(source, 1);
    RULE.assert_fix_explanation_contains(source, "find");
    RULE.assert_fix_explanation_contains(source, "case-insensitive");
}

#[test]
fn fix_description_mentions_where() {
    let source = r#"^grep "pattern" file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fix_explanation_contains(source, "where");
}

#[test]
fn handles_variable_pattern() {
    let source = "^grep $pattern file.txt";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ "$pattern""#);
}

#[test]
fn handles_variable_filename() {
    let source = r#"^grep "error" $logfile"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open $logfile");
}

#[test]
fn handles_escaped_quotes_in_pattern() {
    let source = r#"^grep "error \"critical\"" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ "error "critical"""#);
}

#[test]
fn fix_description_mentions_structured_data() {
    let source = r#"^grep "error" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_grep_multiple_files() {
    let source = r#"^grep "pattern" file1.txt file2.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open file1.txt file2.txt | lines | where $it =~ "pattern""#,
    );
}

#[test]
fn fix_grep_no_file_uses_find() {
    let source = r#"^grep "test""#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"find "test""#);
    RULE.assert_fix_explanation_contains(source, "default");
}

#[test]
fn fix_preserves_pattern() {
    let source = r#"^grep "complex[0-9]+" file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ "complex[0-9]+""#);
}

#[test]
fn fix_handles_single_quotes() {
    let source = r"^grep 'pattern' file.txt";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ "pattern""#);
}

#[test]
fn fix_combined_line_number_and_count() {
    let source = r#"^grep -nc "pattern" file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "enumerate");
    RULE.assert_fixed_contains(source, "length");
}

#[test]
fn fix_invert_with_count() {
    let source = r#"^grep -vc "debug" file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it !~ "debug""#);
    RULE.assert_fixed_contains(source, "length");
}

#[test]
fn fix_explanation_for_case_insensitive_default() {
    let source = r#"^grep "pattern""#;
    RULE.assert_count(source, 1);
    RULE.assert_fix_explanation_contains(source, "case-insensitive by default");
}
