use super::RULE;

#[test]
fn fix_simple_grep_to_find() {
    RULE.assert_fixed_is(r#"^grep "pattern""#, r#"find "pattern""#);
}

#[test]
fn fix_ripgrep_to_find() {
    RULE.assert_fixed_is(r#"^rg "pattern""#, r#"find "pattern""#);
}

#[test]
fn fix_grep_with_file_to_where() {
    RULE.assert_fixed_is(
        r#"^grep "error" logs.txt"#,
        r#"open logs.txt | lines | where $it =~ "error""#,
    );
}

#[test]
fn fix_grep_case_insensitive_drops_redundant_flag() {
    RULE.assert_fixed_is(
        r#"^grep -i "warning" logs.txt"#,
        r#"open logs.txt | lines | where $it =~ "warning""#,
    );
}

#[test]
fn fix_grep_invert_match() {
    RULE.assert_fixed_is(
        r#"^grep -v "debug" app.log"#,
        r#"open app.log | lines | where $it !~ "debug""#,
    );
}

#[test]
fn fix_grep_line_number() {
    RULE.assert_fixed_is(
        r#"^grep -n "TODO" source.rs"#,
        r#"open source.rs | lines | enumerate | where $it =~ "TODO""#,
    );
}

#[test]
fn fix_grep_count() {
    RULE.assert_fixed_is(
        r#"^grep -c "error" logs.txt"#,
        r#"open logs.txt | lines | where $it =~ "error" | length"#,
    );
}

#[test]
fn fix_grep_combined_line_number_and_count() {
    RULE.assert_fixed_is(
        r#"^grep -nc "pattern" file.txt"#,
        r#"open file.txt | lines | enumerate | where $it =~ "pattern" | length"#,
    );
}

#[test]
fn fix_grep_multiple_files() {
    RULE.assert_fixed_is(
        r#"^grep "pattern" file1.txt file2.txt"#,
        r#"open file1.txt file2.txt | lines | where $it =~ "pattern""#,
    );
}

#[test]
fn fix_keeps_variable_pattern() {
    RULE.assert_fixed_is(
        "^grep $pattern file.txt",
        r#"open file.txt | lines | where $it =~ "$pattern""#,
    );
}

#[test]
fn fix_keeps_variable_filename() {
    RULE.assert_fixed_is(
        r#"^grep "error" $logfile"#,
        r#"open $logfile | lines | where $it =~ "error""#,
    );
}

#[test]
fn fix_normalizes_single_quoted_pattern() {
    RULE.assert_fixed_is(
        r"^grep 'pattern' file.txt",
        r#"open file.txt | lines | where $it =~ "pattern""#,
    );
}

#[test]
fn no_fix_without_pattern() {
    RULE.assert_detects_without_fix("^grep");
    RULE.assert_detects_without_fix("^grep -i");
}
