use super::RULE;

#[test]
fn ignore_statements_at_same_indent() {
    RULE.assert_ignores("^echo first\n^echo second");
}

#[test]
fn ignore_indented_block_body() {
    RULE.assert_ignores("def build [] {\n    ^make\n    ^make install\n}");
}

#[test]
fn ignore_multiline_pipeline() {
    RULE.assert_ignores("ls\n    | where size > 1kb\n    | first");
}

#[test]
fn ignore_unindented_multiline_string() {
    RULE.assert_ignores("^echo 'first line\nsecond line'");
}

#[test]
fn ignore_raw_string_with_indented_lines() {
    RULE.assert_ignores("^echo r#'first\n    second'#");
}

#[test]
fn ignore_statement_after_bash_line_continuation() {
    RULE.assert_ignores("^grep -a \\\n    pattern");
}
