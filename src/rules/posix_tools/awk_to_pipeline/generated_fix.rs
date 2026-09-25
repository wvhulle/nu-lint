use super::RULE;

#[test]
fn fix_awk_print_first_field() {
    RULE.assert_fixed_is(
        r#"^awk '{print $1}' input.txt"#,
        r#"open --raw input.txt | lines | split column " " | get column1"#,
    );
}

#[test]
fn fix_awk_with_colon_separator() {
    RULE.assert_fixed_is(
        r#"^awk -F: '{print $1}' /etc/passwd"#,
        "open --raw /etc/passwd | lines | split column : | get column1",
    );
}

#[test]
fn fix_awk_with_comma_separator() {
    RULE.assert_fixed_is(
        r#"^awk -F, '{print $2}' data.csv"#,
        "open --raw data.csv | lines | split column , | get column2",
    );
}

#[test]
fn fix_awk_with_separate_separator_flag() {
    RULE.assert_fixed_is(
        r#"^awk -F ":" '{print $1}' file"#,
        "open --raw file | lines | split column : | get column1",
    );
}

#[test]
fn fix_awk_with_pattern_filter() {
    RULE.assert_fixed_is(
        r#"^awk '/error/' logfile"#,
        r#"open --raw logfile | lines | where $it =~ "error""#,
    );
}

#[test]
fn fix_awk_pattern_and_print() {
    RULE.assert_fixed_is(
        r#"^awk '/warning/ {print $1}' logs.txt"#,
        r#"open --raw logs.txt | lines | where $it =~ "warning" | split column " " | get column1"#,
    );
}

#[test]
fn fix_awk_without_file_reads_pipeline_input() {
    RULE.assert_fixed_is(
        r#"^awk '{print $1}'"#,
        r#"lines | split column " " | get column1"#,
    );
}

#[test]
fn fix_gawk_and_mawk_same_as_awk() {
    RULE.assert_fixed_is(
        r#"^gawk '{print $1}' file.txt"#,
        r#"open --raw file.txt | lines | split column " " | get column1"#,
    );
    RULE.assert_fixed_is(
        r#"^mawk '{print $1}' file.txt"#,
        r#"open --raw file.txt | lines | split column " " | get column1"#,
    );
}

#[test]
fn fix_keeps_variable_filename() {
    RULE.assert_fixed_is(
        r#"^awk '{print $1}' $file"#,
        r#"open --raw $file | lines | split column " " | get column1"#,
    );
}

#[test]
fn no_fix_without_program() {
    RULE.assert_detects_without_fix("^awk");
    RULE.assert_detects_without_fix("^awk -F:");
}

#[test]
fn no_fix_for_identity_program() {
    RULE.assert_detects_without_fix(r#"^awk '{print}' file.txt"#);
}

#[test]
fn no_fix_when_field_reference_is_not_translated() {
    RULE.assert_detects_without_fix(r#"^awk '{print NR, $1}' file.txt"#);
    RULE.assert_detects_without_fix(r#"^awk '{print $1, $2}' file.txt"#);
}
