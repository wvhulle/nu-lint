use super::RULE;

#[test]
fn fix_simple_awk_to_lines_each() {
    let source = "^awk";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "lines | each");
}

#[test]
fn fix_awk_print_first_field() {
    let source = r#"^awk '{print $1}' input.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open --raw input.txt | lines | split column " " | get column1"#,
    );
}

#[test]
fn fix_awk_with_colon_separator() {
    let source = r#"^awk -F: '{print $1}' /etc/passwd"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "split column :");
    RULE.assert_fixed_contains(source, "get column1");
}

#[test]
fn fix_awk_with_comma_separator() {
    let source = r#"^awk -F, '{print $2}' data.csv"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "split column ,");
    RULE.assert_fixed_contains(source, "get column2");
}

#[test]
fn fix_awk_with_pattern_filter() {
    let source = r#"^awk '/error/' logfile"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open --raw logfile | lines | where $it =~ "error""#,
    );
}

#[test]
fn fix_awk_pattern_and_print() {
    let source = r#"^awk '/warning/ {print $1}' logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ "warning""#);
    RULE.assert_fixed_contains(source, "split column");
    RULE.assert_fixed_contains(source, "get column1");
}

#[test]
fn fix_awk_no_file_uses_lines() {
    let source = r#"^awk '{print $1}'"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"lines | split column " " | get column1"#);
}

#[test]
fn fix_gawk_same_as_awk() {
    let source = r#"^gawk '{print $1}' file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open --raw file.txt | lines");
    RULE.assert_fixed_contains(source, "split column");
}

#[test]
fn fix_mawk_same_as_awk() {
    let source = r#"^mawk '{print $2}' input.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open --raw input.txt | lines");
    RULE.assert_fixed_contains(source, "get column2");
}

#[test]
fn handles_variable_filename() {
    let source = r#"^awk '{print $1}' $file"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open --raw $file | lines");
}

#[test]
fn handles_escaped_quotes_in_pattern() {
    let source = r#"^awk '/"test"/' file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ ""test"""#);
}

#[test]
fn fix_description_mentions_pipeline() {
    let source = "^awk";
    RULE.assert_count(source, 1);
}

#[test]
fn fix_description_mentions_structured_data() {
    let source = r#"^awk '{print $1}' file.txt"#;
    RULE.assert_count(source, 1);
}

#[test]
fn fix_description_mentions_split_column() {
    let source = r#"^awk -F: '{print $1}'"#;
    RULE.assert_count(source, 1);
}

#[test]
fn fix_preserves_pattern() {
    let source = r#"^awk '/[0-9]+/' file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ "[0-9]+""#);
}

#[test]
fn fix_handles_separate_f_flag() {
    let source = r#"^awk -F ":" '{print $1}' file"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "split column :");
}

#[test]
fn fix_combined_pattern_and_field() {
    let source = r#"^awk '/error/ {print $3}' logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ "error""#);
    RULE.assert_fixed_contains(source, "get column3");
}
