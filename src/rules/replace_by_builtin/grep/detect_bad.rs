use super::rule;

#[test]
fn detect_simple_grep() {
    rule().assert_detects(r#"^grep "pattern""#);
}

#[test]
fn detect_grep_with_file() {
    rule().assert_detects(r#"^grep "error" logs.txt"#);
}

#[test]
fn detect_grep_case_insensitive() {
    rule().assert_detects(r#"^grep -i "warning" file.txt"#);
}

#[test]
fn detect_grep_invert_match() {
    rule().assert_detects(r#"^grep -v "debug" app.log"#);
}

#[test]
fn detect_grep_line_number() {
    rule().assert_detects(r#"^grep -n "TODO" source.rs"#);
}

#[test]
fn detect_grep_count() {
    rule().assert_detects(r#"^grep -c "error" logs.txt"#);
}

#[test]
fn detect_grep_files_with_matches() {
    rule().assert_detects(r#"^grep -l "pattern" *.txt"#);
}

#[test]
fn detect_grep_extended_regex() {
    rule().assert_detects(r#"^grep -E "pattern+" file.txt"#);
}

#[test]
fn detect_grep_fixed_strings() {
    rule().assert_detects(r#"^grep -F "literal.string" file.txt"#);
}

#[test]
fn detect_grep_recursive() {
    rule().assert_detects(r#"^grep -r "TODO" ."#);
}

#[test]
fn detect_ripgrep() {
    rule().assert_detects(r#"^rg "pattern""#);
}

#[test]
fn detect_ripgrep_with_file() {
    rule().assert_detects(r#"^rg "error" logs.txt"#);
}

#[test]
fn detect_grep_combined_flags() {
    let bad_codes = vec![
        r#"^grep -in "pattern" file.txt"#,
        r#"^grep -rn "TODO" src/"#,
        r#"^grep -vic "warning" logs.txt"#,
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_grep_in_pipeline() {
    rule().assert_detects(r#"cat file.txt | ^grep "pattern""#);
}

#[test]
fn detect_grep_in_function() {
    let bad_code = r"
def search-logs [pattern] {
    ^grep $pattern logs.txt
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_multiple_grep_uses() {
    let bad_code = r#"
^grep "error" file1.txt
^grep "warning" file2.txt
"#;
    rule().assert_violation_count_exact(bad_code, 2);
}

#[test]
fn detect_grep_with_context() {
    let bad_codes = vec![
        r#"^grep -A 3 "pattern" file.txt"#,
        r#"^grep -B 2 "pattern" file.txt"#,
        r#"^grep -C 5 "pattern" file.txt"#,
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_grep_no_file() {
    rule().assert_detects(r#"^grep "pattern""#);
}

#[test]
fn detect_grep_multiple_files() {
    rule().assert_detects(r#"^grep "pattern" file1.txt file2.txt"#);
}

#[test]
fn detect_grep_with_glob() {
    rule().assert_detects(r#"^grep "pattern" *.log"#);
}

#[test]
fn detect_grep_long_options() {
    let bad_codes = vec![
        r#"^grep --ignore-case "pattern" file.txt"#,
        r#"^grep --invert-match "debug" logs.txt"#,
        r#"^grep --line-number "TODO" source.rs"#,
        r#"^grep --count "error" logs.txt"#,
        r#"^grep --files-with-matches "pattern" *.txt"#,
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_grep_in_subexpression() {
    let bad_code = r#"
if (^grep -c "error" logs.txt) > 0 {
    print "Found errors"
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn detect_grep_in_closure() {
    let bad_code = r#"
ls | each { |file|
    ^grep "TODO" $file.name
}
"#;
    rule().assert_detects(bad_code);
}
