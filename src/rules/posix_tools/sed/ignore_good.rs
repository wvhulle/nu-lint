use super::RULE;

#[test]
fn test_ignore_builtin_str_replace() {
    let good_code = "str replace 'old' 'new'";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_str_replace_all() {
    let good_code = "str replace --all 'old' 'new'";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_open_pipe_str_replace() {
    let good_code = "open file.txt | str replace 'pattern' 'replacement'";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_open_str_replace_save() {
    let good_code = "open file.txt | str replace 'old' 'new' | save -f file.txt";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_lines_where_filter() {
    let good_code = "lines | where $it !~ 'pattern'";
    RULE.assert_ignores(good_code);
}
