use super::rule;

#[test]
fn fix_simple_cat_to_open_raw() {
    let source = "^cat file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open --raw file.txt");
}

#[test]
fn fix_description_mentions_structured() {
    let source = "^cat file.txt";
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_multiple_files_to_each_open() {
    let source = "^cat file1.txt file2.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(
        source,
        "[file1.txt file2.txt] | each {|f| open --raw $f} | str join",
    );
    rule().assert_fix_explanation_contains(source, "each");
    rule().assert_fix_explanation_contains(source, "multiple");
}

#[test]
fn fix_tac_to_open_raw() {
    let source = "^tac file.log";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open --raw file.log");
}

#[test]
fn fix_more_to_open_raw() {
    let source = "^more documentation.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open --raw documentation.txt");
}

#[test]
fn fix_less_to_open_raw() {
    let source = "^less output.log";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open --raw output.log");
}

#[test]
fn fix_number_lines_flag() {
    let source = "^cat -n file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open --raw file.txt | lines | enumerate");
    rule().assert_fix_explanation_contains(source, "enumerate");
    rule().assert_fix_explanation_contains(source, "-n");
}

#[test]
fn fix_number_nonblank_flag() {
    let source = "^cat -b file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(
        source,
        "open --raw file.txt | lines | enumerate | where $it.item != \"\"",
    );
    rule().assert_fix_explanation_contains(source, "non-blank");
    rule().assert_fix_explanation_contains(source, "enumerate");
}

#[test]
fn fix_show_ends_flag() {
    let source = "^cat -E file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open --raw file.txt | lines");
    rule().assert_fix_explanation_contains(source, "-E");
    rule().assert_fix_explanation_contains(source, "line endings");
}

#[test]
fn fix_show_tabs_flag() {
    let source = "^cat -T file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open --raw file.txt | lines");
    rule().assert_fix_explanation_contains(source, "-T");
    rule().assert_fix_explanation_contains(source, "tabs");
}

#[test]
fn fix_show_all_flag() {
    let source = "^cat -A file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open --raw file.txt | lines");
    rule().assert_fix_explanation_contains(source, "-E");
}

#[test]
fn fix_combines_number_with_multiple_files() {
    let source = "^cat -n file1.txt file2.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "lines");
    rule().assert_replacement_contains(source, "enumerate");
}

#[test]
fn fix_long_number_option() {
    let source = "^cat --number file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "enumerate");
}

#[test]
fn fix_long_number_nonblank_option() {
    let source = "^cat --number-nonblank file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "enumerate");
    rule().assert_replacement_contains(source, "where");
}

#[test]
fn fix_long_show_ends_option() {
    let source = "^cat --show-ends file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "lines");
}

#[test]
fn fix_long_show_tabs_option() {
    let source = "^cat --show-tabs file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "lines");
}

#[test]
fn fix_long_show_all_option() {
    let source = "^cat --show-all file.txt";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "lines");
}

#[test]
fn fix_preserves_filename() {
    let source = "^cat my-complex-filename.log";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "my-complex-filename.log");
}

#[test]
fn fix_description_explains_open_benefits() {
    let source = "^cat file.txt";
    rule().assert_fix_explanation_contains(source, "open");
}
