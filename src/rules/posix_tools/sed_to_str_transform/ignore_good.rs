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
fn test_ignore_sed_delete_command() {
    let good_code = "^sed '/pattern/d'";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_sed_print_command() {
    let good_code = "^sed '/pattern/p'";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_sed_with_script_file() {
    let good_code = "^sed -f script.sed file.txt";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_sed_with_address_range() {
    let good_code = "^sed '1,10s/old/new/'";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_sed_with_multiple_commands() {
    let good_code = "^sed 's/a/b/;s/c/d/'";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_sed_quiet_mode() {
    let good_code = "^sed -n 's/pattern/replacement/p'";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_sed_without_substitution() {
    let good_code = "^sed";
    RULE.assert_ignores(good_code);
}
