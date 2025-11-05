use crate::rules::prefer_builtin::cat::rule;

#[test]
fn replaces_simple_cat_with_open_raw() {
    let source = "^cat file.txt";
    rule().assert_fix_contains(source, "open --raw file.txt");
    rule().assert_fix_description_contains(source, "auto-parse");
}

#[test]
fn suggests_open_for_first_file_when_multiple() {
    let source = "^cat file1.txt file2.txt";
    rule().assert_fix_contains(source, "[file1.txt file2.txt] | each {|f| open --raw $f} | str join");
    rule().assert_fix_description_contains(source, "each");
    rule().assert_fix_description_contains(source, "multiple");
}

#[test]
fn handles_structured_files() {
    let source = "^cat config.json";
    rule().assert_fix_contains(source, "open --raw config.json");
}

#[test]
fn ignores_builtin_open() {
    let source = "open file.txt";
    rule().assert_ignores(source);
}
