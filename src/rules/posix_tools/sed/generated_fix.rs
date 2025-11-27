use super::rule;

#[test]
fn test_fix_sed_substitution_to_str_replace() {
    let bad_code = r"^sed 's/foo/bar/'";
    rule().assert_replacement_contains(bad_code, "str replace 'foo' 'bar'");
    rule().assert_fix_explanation_contains(bad_code, "str replace");
}

#[test]
fn test_fix_sed_global_flag_to_str_replace_all() {
    let bad_code = r"^sed 's/old/new/g'";
    rule().assert_replacement_contains(bad_code, "str replace --all 'old' 'new'");
    rule().assert_fix_explanation_contains(bad_code, "--all");
    rule().assert_fix_explanation_contains(bad_code, "/g");
}

#[test]
fn test_fix_sed_with_file_to_open_pipe() {
    let bad_code = r"^sed 's/pattern/replacement/' file.txt";
    rule().assert_replacement_contains(
        bad_code,
        "open file.txt | str replace 'pattern' 'replacement'",
    );
    rule().assert_fix_explanation_contains(bad_code, "open");
}

#[test]
fn test_fix_sed_inplace_to_open_save() {
    let bad_code = r"^sed -i 's/old/new/' file.txt";
    rule().assert_replacement_contains(
        bad_code,
        "open file.txt | str replace 'old' 'new' | save -f file.txt",
    );
    rule().assert_fix_explanation_contains(bad_code, "in-place");
    rule().assert_fix_explanation_contains(bad_code, "save");
}

#[test]
fn test_fix_sed_inplace_global_to_open_replace_all_save() {
    let bad_code = r"^sed -i 's/foo/bar/g' config.ini";
    rule().assert_replacement_contains(
        bad_code,
        "open config.ini | str replace --all 'foo' 'bar' | save -f config.ini",
    );
    rule().assert_fix_explanation_contains(bad_code, "--all");
}

#[test]
fn test_fix_sed_delete_to_lines_where() {
    let bad_code = r"^sed '/pattern/d'";
    rule().assert_replacement_contains(bad_code, "lines | where $it !~ 'pattern'");
    rule().assert_fix_explanation_contains(bad_code, "where");
}

#[test]
fn test_fix_sed_combined_inplace_flags() {
    let bad_code = r"^sed -ie 's/test/prod/g' app.conf";
    rule().assert_replacement_contains(bad_code, "open");
    rule().assert_replacement_contains(bad_code, "save");
}

#[test]
fn test_fix_sed_default_suggestion() {
    let bad_code = r"^sed";
    rule().assert_fix_explanation_contains(bad_code, "str replace");
}

#[test]
fn test_fix_gsed_to_str_replace() {
    let bad_code = r"^gsed 's/foo/bar/'";
    rule().assert_replacement_contains(bad_code, "str replace");
}
