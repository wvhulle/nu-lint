use super::RULE;

#[test]
fn fix_substitution_to_str_replace() {
    RULE.assert_fixed_is("^sed 's/foo/bar/'", "str replace 'foo' 'bar'");
}

#[test]
fn fix_gsed_to_str_replace() {
    RULE.assert_fixed_is("^gsed 's/foo/bar/'", "str replace 'foo' 'bar'");
}

#[test]
fn fix_global_flag_to_str_replace_all() {
    RULE.assert_fixed_is("^sed 's/old/new/g'", "str replace --all 'old' 'new'");
}

#[test]
fn fix_with_file_to_open_pipe() {
    RULE.assert_fixed_is(
        "^sed 's/pattern/replacement/' file.txt",
        "open file.txt | str replace 'pattern' 'replacement'",
    );
}

#[test]
fn fix_inplace_to_open_save() {
    RULE.assert_fixed_is(
        "^sed -i 's/old/new/' file.txt",
        "open file.txt | str replace 'old' 'new' | save -f file.txt",
    );
}

#[test]
fn fix_inplace_global_to_open_replace_all_save() {
    RULE.assert_fixed_is(
        "^sed -i 's/foo/bar/g' config.ini",
        "open config.ini | str replace --all 'foo' 'bar' | save -f config.ini",
    );
}

#[test]
fn fix_extended_regex() {
    RULE.assert_fixed_is(
        "^sed -E 's/[0-9]+/NUM/'",
        "str replace --regex '[0-9]+' 'NUM'",
    );
}

#[test]
fn fix_expression_flag() {
    RULE.assert_fixed_is("^sed -e 's/a/b/'", "str replace 'a' 'b'");
}

#[test]
fn fix_combined_flags() {
    RULE.assert_fixed_is(
        "^sed -Ei 's/pattern/repl/g' file.txt",
        "open file.txt | str replace --all --regex 'pattern' 'repl' | save -f file.txt",
    );
}

#[test]
fn fix_keeps_variable_filename() {
    RULE.assert_fixed_is(
        "^sed 's/foo/bar/' $file",
        "open $file | str replace 'foo' 'bar'",
    );
}

#[test]
fn ignores_sed_without_substitution_script() {
    RULE.assert_ignores("^sed -i");
    RULE.assert_ignores("^sed '1d' file.txt");
    RULE.assert_ignores("^sed 's|only-one-delimiter' file.txt");
}
