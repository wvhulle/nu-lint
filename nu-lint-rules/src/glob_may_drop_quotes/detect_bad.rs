use super::RULE;

#[test]
fn detects_quoted_asterisk_glob_to_ls() {
    let code = r#"ls "*.txt""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_quoted_question_mark_glob_to_ls() {
    let code = r#"ls "file?.rs""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_quoted_glob_to_rm() {
    let code = r#"rm "*.log""#;
    RULE.assert_detects(code);
}

// NOTE: The `glob` command expects a string parameter (not a glob pattern).
// It processes the pattern internally, so quotes should NOT be removed.
// This is different from `ls`, `rm`, etc. which expect actual glob patterns.
// Test removed as the current behavior (not flagging) is correct.

#[test]
fn detects_quoted_glob_with_single_quotes() {
    let code = r#"ls '*.txt'"#;
    RULE.assert_detects(code);
}

#[test]
fn detects_complex_glob_pattern() {
    let code = r#"ls "src/**/*.rs""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_glob_with_question_mark() {
    let code = r#"ls "test?.txt""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_glob_in_cp_source() {
    let code = r#"cp "*.txt" dest/"#;
    RULE.assert_detects(code);
}

#[test]
fn detects_glob_in_mv_source() {
    let code = r#"mv "*.txt" dest/"#;
    RULE.assert_detects(code);
}

#[test]
fn detects_multiple_glob_args() {
    // Only the first arg to rm is glob, but we detect it
    let code = r#"rm "*.txt""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_glob_with_spaces() {
    let code = r#"ls "my files *""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_glob_with_spaces_and_wildcards() {
    let code = r#"ls "test files/*.txt""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_glob_with_special_chars() {
    let code = r#"ls "path|with|pipes*""#;
    RULE.assert_detects(code);
}
