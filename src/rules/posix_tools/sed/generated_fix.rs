use super::RULE;

#[test]
fn test_fix_sed_substitution_to_str_replace() {
    let bad_code = r"^sed 's/foo/bar/'";
    RULE.assert_fixed_contains(bad_code, "str replace 'foo' 'bar'");
}

#[test]
fn test_fix_sed_global_flag_to_str_replace_all() {
    let bad_code = r"^sed 's/old/new/g'";
    RULE.assert_fixed_contains(bad_code, "str replace --all 'old' 'new'");
}

#[test]
fn test_fix_sed_with_file_to_open_pipe() {
    let bad_code = r"^sed 's/pattern/replacement/' file.txt";
    RULE.assert_fixed_contains(
        bad_code,
        "open file.txt | str replace 'pattern' 'replacement'",
    );
}

#[test]
fn test_fix_sed_inplace_to_open_save() {
    let bad_code = r"^sed -i 's/old/new/' file.txt";
    RULE.assert_fixed_contains(
        bad_code,
        "open file.txt | str replace 'old' 'new' | save -f file.txt",
    );
}

#[test]
fn test_fix_sed_inplace_global_to_open_replace_all_save() {
    let bad_code = r"^sed -i 's/foo/bar/g' config.ini";
    RULE.assert_fixed_contains(
        bad_code,
        "open config.ini | str replace --all 'foo' 'bar' | save -f config.ini",
    );
}

#[test]
fn test_fix_sed_extended_regex() {
    let bad_code = r"^sed -E 's/[0-9]+/NUM/'";
    RULE.assert_fixed_contains(bad_code, "str replace --regex '[0-9]+' 'NUM'");
}

#[test]
fn test_fix_sed_expression_flag() {
    let bad_code = r"^sed -e 's/a/b/'";
    RULE.assert_fixed_contains(bad_code, "str replace 'a' 'b'");
}

#[test]
fn test_fix_sed_combined_flags() {
    let bad_code = r"^sed -Ei 's/pattern/repl/g' file.txt";
    RULE.assert_fixed_contains(bad_code, "--all");
    RULE.assert_fixed_contains(bad_code, "--regex");
    RULE.assert_fixed_contains(bad_code, "save -f");
}

#[test]
fn test_fix_gsed_to_str_replace() {
    let bad_code = r"^gsed 's/foo/bar/'";
    RULE.assert_fixed_contains(bad_code, "str replace 'foo' 'bar'");
}

#[test]
fn handles_variable_filename() {
    let bad_code = r"^sed 's/foo/bar/' $file";
    RULE.assert_fixed_contains(bad_code, "open $file | str replace 'foo' 'bar'");
}

#[test]
fn handles_escaped_quotes_in_replacement() {
    let bad_code = r#"^sed "s/old/\"new\"/" file.txt"#;
    RULE.assert_count(bad_code, 1);
}
