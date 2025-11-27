use super::rule;

#[test]
fn test_ignore_builtin_str_replace() {
    let good_code = "str replace 'old' 'new'";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_str_replace_all() {
    let good_code = "str replace --all 'old' 'new'";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_open_pipe_str_replace() {
    let good_code = "open file.txt | str replace 'pattern' 'replacement'";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_open_str_replace_save() {
    let good_code = "open file.txt | str replace 'old' 'new' | save -f file.txt";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_lines_where_filter() {
    let good_code = "lines | where $it !~ 'pattern'";
    rule().assert_ignores(good_code);
}
