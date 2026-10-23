use super::RULE;

#[test]
fn fix_piped_grep() {
    RULE.assert_fixed_is(
        r#"^cat log.txt | ^grep "error""#,
        r#"^cat log.txt | lines | where $it =~ "error""#,
    );
}

#[test]
fn fix_piped_ripgrep_keeps_regex() {
    RULE.assert_fixed_is(r"$text | ^rg 'a+b'", r"$text | lines | where $it =~ 'a+b'");
}

#[test]
fn fix_bare_pattern_is_quoted() {
    RULE.assert_fixed_is(
        r"$text | ^grep error",
        r"$text | lines | where $it =~ 'error'",
    );
}

#[test]
fn fix_single_file_opens_raw() {
    RULE.assert_fixed_is(
        r#"^grep "error" data.json"#,
        r#"open --raw data.json | lines | where $it =~ "error""#,
    );
}

#[test]
fn fix_keeps_quoted_file_name() {
    RULE.assert_fixed_is(
        r#"^grep 'x' "my logs.txt""#,
        r#"open --raw "my logs.txt" | lines | where $it =~ 'x'"#,
    );
}

#[test]
fn fix_keeps_variable_file() {
    RULE.assert_fixed_is(
        r#"let logfile = "a.log"; ^grep "error" $logfile"#,
        r#"let logfile = "a.log"; open --raw $logfile | lines | where $it =~ "error""#,
    );
}

#[test]
fn fix_ignore_case_regex() {
    RULE.assert_fixed_is(
        r#"$text | ^grep -i "warning""#,
        r"$text | lines | where $it =~ '(?i)warning'",
    );
}

#[test]
fn fix_ignore_case_dynamic_extended_regex() {
    RULE.assert_fixed_is(
        r#"let pattern = "x"; $text | ^grep -iE $pattern"#,
        r#"let pattern = "x"; $text | lines | where $it =~ ('(?i)' + $pattern)"#,
    );
}

#[test]
fn fix_invert_match() {
    RULE.assert_fixed_is(
        r#"$text | ^grep -v "debug""#,
        r#"$text | lines | where $it !~ "debug""#,
    );
}

#[test]
fn fix_count() {
    RULE.assert_fixed_is(
        r#"^grep -c "error" logs.txt"#,
        r#"open --raw logs.txt | lines | where $it =~ "error" | length"#,
    );
}

#[test]
fn fix_fixed_strings() {
    RULE.assert_fixed_is(
        r#"$text | ^grep -F "a.b""#,
        r#"$text | lines | where ($it | str contains "a.b")"#,
    );
}

#[test]
fn fix_fixed_strings_ignore_case_inverted() {
    RULE.assert_fixed_is(
        r#"$text | ^grep -Fiv "a.b""#,
        r#"$text | lines | where not ($it | str contains --ignore-case "a.b")"#,
    );
}

#[test]
fn fix_extended_regex() {
    RULE.assert_fixed_is(
        r#"$text | ^grep -E "a+|b""#,
        r#"$text | lines | where $it =~ "a+|b""#,
    );
}

#[test]
fn fix_basic_regex_shared_with_rust_regex() {
    RULE.assert_fixed_is(
        r"$text | ^grep '^a.*\.rs$'",
        r"$text | lines | where $it =~ '^a.*\.rs$'",
    );
}
