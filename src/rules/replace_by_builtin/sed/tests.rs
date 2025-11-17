use crate::rules::replace_by_builtin::sed::rule;

#[test]
fn converts_sed_substitution_to_str_replace() {
    let source = r"^sed 's/foo/bar/'";
    rule().assert_replacement_contains(source, "str replace 'foo' 'bar'");
    rule().assert_fix_explanation_contains(source, "str replace");
}

#[test]
fn converts_sed_global_flag_to_str_replace_all() {
    let source = r"^sed 's/old/new/g'";
    rule().assert_replacement_contains(source, "str replace --all 'old' 'new'");
    rule().assert_fix_explanation_contains(source, "--all");
    rule().assert_fix_explanation_contains(source, "/g");
}

#[test]
fn converts_sed_with_file_to_open_pipe_str_replace() {
    let source = r"^sed 's/pattern/replacement/' file.txt";
    rule().assert_replacement_contains(
        source,
        "open file.txt | str replace 'pattern' 'replacement'",
    );
    rule().assert_fix_explanation_contains(source, "open");
}

#[test]
fn converts_sed_inplace_to_open_replace_save() {
    let source = r"^sed -i 's/old/new/' file.txt";
    rule().assert_replacement_contains(
        source,
        "open file.txt | str replace 'old' 'new' | save -f file.txt",
    );
    rule().assert_fix_explanation_contains(source, "in-place");
    rule().assert_fix_explanation_contains(source, "save");
}

#[test]
fn converts_sed_inplace_global_to_open_replace_all_save() {
    let source = r"^sed -i 's/foo/bar/g' config.ini";
    rule().assert_replacement_contains(
        source,
        "open config.ini | str replace --all 'foo' 'bar' | save -f config.ini",
    );
    rule().assert_fix_explanation_contains(source, "--all");
}

#[test]
fn converts_sed_delete_to_lines_where_not_match() {
    let source = r"^sed '/pattern/d'";
    rule().assert_replacement_contains(source, "lines | where $it !~ 'pattern'");
    rule().assert_fix_explanation_contains(source, "where");
}

#[test]
fn converts_sed_combined_inplace_flags() {
    let source = r"^sed -ie 's/test/prod/g' app.conf";
    rule().assert_replacement_contains(source, "open");
    rule().assert_replacement_contains(source, "save");
}

#[test]
fn provides_default_suggestion_for_complex_sed() {
    let source = r"^sed";
    rule().assert_fix_explanation_contains(source, "str replace");
}

#[test]
fn ignores_builtin_str_replace() {
    let source = "str replace 'old' 'new'";
    rule().assert_ignores(source);
}

#[test]
fn converts_gsed_to_str_replace() {
    let source = r"^gsed 's/foo/bar/'";
    rule().assert_replacement_contains(source, "str replace");
}
