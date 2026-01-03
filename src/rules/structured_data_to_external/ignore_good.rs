use super::RULE;

#[test]
fn ignore_string_to_external() {
    RULE.assert_ignores("'hello world' | ^cat");
}

#[test]
fn ignore_to_json_before_external() {
    RULE.assert_ignores("ls | to json | ^cat");
}

#[test]
fn ignore_to_text_before_external() {
    RULE.assert_ignores("ls | to text | ^cat");
}

#[test]
fn ignore_to_csv_before_external() {
    RULE.assert_ignores("ls | to csv | ^cat");
}

#[test]
fn ignore_str_join_before_external() {
    RULE.assert_ignores("[1, 2, 3] | str join ',' | ^cat");
}

#[test]
fn ignore_open_text_file_to_external() {
    RULE.assert_ignores("open file.txt | ^grep test");
}

#[test]
fn ignore_single_external_call() {
    RULE.assert_ignores("^ls -la");
}

#[test]
fn ignore_string_interpolation_to_external() {
    RULE.assert_ignores("$'hello (date now)' | ^cat");
}

#[test]
fn ignore_into_string_before_external() {
    RULE.assert_ignores("42 | into string | ^cat");
}

#[test]
fn ignore_builtin_after_table() {
    // This shouldn't trigger because 'cat' without ^ is a builtin
    RULE.assert_ignores("ls | first 5");
}
