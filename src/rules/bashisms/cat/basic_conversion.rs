use crate::rules::bashisms::cat::rule;

#[test]
fn converts_cat_single_file_to_open_raw() {
    let source = "^cat file.txt";
    rule().assert_replacement_contains(source, "open --raw file.txt");
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn converts_cat_multiple_files_to_each_open() {
    let source = "^cat file1.txt file2.txt";
    rule().assert_replacement_contains(
        source,
        "[file1.txt file2.txt] | each {|f| open --raw $f} | str join",
    );
    rule().assert_fix_explanation_contains(source, "each");
    rule().assert_fix_explanation_contains(source, "multiple");
}

#[test]
fn converts_tac_to_open_raw() {
    let source = "^tac file.log";
    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "open --raw file.log");
}

#[test]
fn converts_more_to_open_raw() {
    let source = "^more documentation.txt";
    rule().assert_replacement_contains(source, "open --raw documentation.txt");
}

#[test]
fn converts_less_to_open_raw() {
    let source = "^less output.log";
    rule().assert_replacement_contains(source, "open --raw output.log");
}

#[test]
fn ignores_builtin_open() {
    let source = "open file.txt";
    rule().assert_ignores(source);
}
