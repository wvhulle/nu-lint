use crate::rules::replace_by_builtin::cat::rule;

#[test]
fn replaces_simple_cat_with_open_raw() {
    let source = "^cat file.txt";
    rule().assert_fix_contains(source, "open --raw file.txt");
    rule().assert_fix_description_contains(source, "structured");
}

#[test]
fn handles_multiple_files() {
    let source = "^cat file1.txt file2.txt";
    rule().assert_fix_contains(
        source,
        "[file1.txt file2.txt] | each {|f| open --raw $f} | str join",
    );
    rule().assert_fix_description_contains(source, "each");
    rule().assert_fix_description_contains(source, "multiple");
}

#[test]
fn handles_structured_files() {
    let source = "^cat config.json";
    rule().assert_fix_contains(source, "open --raw config.json");
}

#[test]
fn detects_tac_command() {
    let source = "^tac file.log";
    rule().assert_detects(source);
    rule().assert_fix_contains(source, "open --raw file.log");
}

#[test]
fn detects_more_command() {
    let source = "^more documentation.txt";
    rule().assert_fix_contains(source, "open --raw documentation.txt");
}

#[test]
fn detects_less_command() {
    let source = "^less output.log";
    rule().assert_fix_contains(source, "open --raw output.log");
}

#[test]
fn ignores_builtin_open() {
    let source = "open file.txt";
    rule().assert_ignores(source);
}
