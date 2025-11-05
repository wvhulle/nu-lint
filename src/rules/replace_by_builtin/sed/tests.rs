use crate::rules::replace_by_builtin::sed::rule;

#[test]
fn replaces_simple_sed_substitution() {
    let source = r"^sed 's/foo/bar/'";
    rule().assert_fix_contains(source, "str replace 'foo' 'bar'");
    rule().assert_fix_description_contains(source, "str replace");
}

#[test]
fn handles_global_flag() {
    let source = r"^sed 's/old/new/g'";
    rule().assert_fix_contains(source, "str replace --all 'old' 'new'");
    rule().assert_fix_description_contains(source, "--all");
    rule().assert_fix_description_contains(source, "/g");
}

#[test]
fn handles_file_input() {
    let source = r"^sed 's/pattern/replacement/' file.txt";
    rule().assert_fix_contains(
        source,
        "open file.txt | str replace 'pattern' 'replacement'",
    );
    rule().assert_fix_description_contains(source, "open");
}

#[test]
fn handles_in_place_editing() {
    let source = r"^sed -i 's/old/new/' file.txt";
    rule().assert_fix_contains(
        source,
        "open file.txt | str replace 'old' 'new' | save -f file.txt",
    );
    rule().assert_fix_description_contains(source, "in-place");
    rule().assert_fix_description_contains(source, "save");
}

#[test]
fn handles_in_place_with_global() {
    let source = r"^sed -i 's/foo/bar/g' config.ini";
    rule().assert_fix_contains(
        source,
        "open config.ini | str replace --all 'foo' 'bar' | save -f config.ini",
    );
    rule().assert_fix_description_contains(source, "--all");
}

#[test]
fn handles_delete_operation() {
    let source = r"^sed '/pattern/d'";
    rule().assert_fix_contains(source, "lines | where $it !~ 'pattern'");
    rule().assert_fix_description_contains(source, "where");
}

#[test]
fn handles_combined_flags() {
    let source = r"^sed -ie 's/test/prod/g' app.conf";
    rule().assert_fix_contains(source, "open");
    rule().assert_fix_contains(source, "save");
}

#[test]
fn provides_default_suggestion_for_complex_sed() {
    let source = r"^sed";
    rule().assert_fix_description_contains(source, "str replace");
}

#[test]
fn ignores_builtin_str_replace() {
    let source = "str replace 'old' 'new'";
    rule().assert_ignores(source);
}

#[test]
fn detects_gsed() {
    let source = r"^gsed 's/foo/bar/'";
    rule().assert_fix_contains(source, "str replace");
}
