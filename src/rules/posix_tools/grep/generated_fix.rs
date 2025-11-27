use super::rule;

#[test]
fn fix_simple_grep_to_find() {
    let source = r#"^grep "pattern""#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"find "pattern""#);
}

#[test]
fn fix_grep_with_file_to_where() {
    let source = r#"^grep "error" logs.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"open logs.txt | lines | where $it =~ "error""#);
}

#[test]
fn fix_grep_case_insensitive() {
    let source = r#"^grep -i "warning" logs.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"open logs.txt | lines | where $it =~ "warning""#);
    rule().assert_fix_explanation_contains(source, "redundant");
    rule().assert_fix_explanation_contains(source, "-i");
}

#[test]
fn fix_grep_invert_match() {
    let source = r#"^grep -v "debug" app.log"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"open app.log | lines | where $it !~ "debug""#);
    rule().assert_fix_explanation_contains(source, "!~");
}

#[test]
fn fix_grep_line_number() {
    let source = r#"^grep -n "TODO" source.rs"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(
        source,
        r#"open source.rs | lines | enumerate | where $it =~ "TODO""#,
    );
    rule().assert_fix_explanation_contains(source, "enumerate");
}

#[test]
fn fix_grep_count() {
    let source = r#"^grep -c "error" logs.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(
        source,
        r#"open logs.txt | lines | where $it =~ "error" | length"#,
    );
    rule().assert_fix_explanation_contains(source, "length");
}

#[test]
fn fix_grep_fixed_strings() {
    let source = r#"^grep -F "literal.string" file.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(
        source,
        r#"open file.txt | lines | where $it | str contains "literal.string""#,
    );
    rule().assert_fix_explanation_contains(source, "str contains");
}

#[test]
fn fix_grep_recursive() {
    let source = r#"^grep -r "TODO" ."#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"open . | lines | where $it =~ "TODO""#);
}

#[test]
fn fix_ripgrep_simple() {
    let source = r#"^rg "pattern""#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"find "pattern""#);
}

#[test]
fn fix_ripgrep_with_file() {
    let source = r#"^rg "error" logs.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"open logs.txt | lines | where $it =~ "error""#);
}

#[test]
fn fix_description_mentions_find() {
    let source = r#"^grep "pattern""#;
    rule().assert_count(source, 1);
    rule().assert_fix_explanation_contains(source, "find");
    rule().assert_fix_explanation_contains(source, "case-insensitive");
}

#[test]
fn fix_description_mentions_where() {
    let source = r#"^grep "pattern" file.txt"#;
    rule().assert_count(source, 1);
    rule().assert_fix_explanation_contains(source, "where");
}

#[test]
fn fix_description_mentions_structured_data() {
    let source = r#"^grep "error" logs.txt"#;
    rule().assert_count(source, 1);
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_grep_multiple_files() {
    let source = r#"^grep "pattern" file1.txt file2.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(
        source,
        r#"open file1.txt file2.txt | lines | where $it =~ "pattern""#,
    );
}

#[test]
fn fix_grep_no_file_uses_find() {
    let source = r#"^grep "test""#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"find "test""#);
    rule().assert_fix_explanation_contains(source, "default");
}

#[test]
fn fix_preserves_pattern() {
    let source = r#"^grep "complex[0-9]+" file.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"where $it =~ "complex[0-9]+""#);
}

#[test]
fn fix_handles_single_quotes() {
    let source = r"^grep 'pattern' file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"where $it =~ "pattern""#);
}

#[test]
fn fix_combined_line_number_and_count() {
    let source = r#"^grep -nc "pattern" file.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "enumerate");
    rule().assert_replacement_contains(source, "length");
}

#[test]
fn fix_invert_with_count() {
    let source = r#"^grep -vc "debug" file.txt"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, r#"where $it !~ "debug""#);
    rule().assert_replacement_contains(source, "length");
}

#[test]
fn fix_explanation_for_case_insensitive_default() {
    let source = r#"^grep "pattern""#;
    rule().assert_count(source, 1);
    rule().assert_fix_explanation_contains(source, "case-insensitive by default");
}
