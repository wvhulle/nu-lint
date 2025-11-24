use crate::rules::bashisms::cat::rule;

#[test]
fn converts_number_lines_flag() {
    let source = "^cat -n file.txt";
    rule().assert_replacement_contains(source, "open --raw file.txt | lines | enumerate");
    rule().assert_fix_explanation_contains(source, "enumerate");
    rule().assert_fix_explanation_contains(source, "-n");
}

#[test]
fn converts_number_nonblank_flag() {
    let source = "^cat -b file.txt";
    rule().assert_replacement_contains(
        source,
        "open --raw file.txt | lines | enumerate | where $it.item != \"\"",
    );
    rule().assert_fix_explanation_contains(source, "non-blank");
    rule().assert_fix_explanation_contains(source, "enumerate");
}

#[test]
fn converts_show_ends_flag() {
    let source = "^cat -E file.txt";
    rule().assert_replacement_contains(source, "open --raw file.txt | lines");
    rule().assert_fix_explanation_contains(source, "-E");
    rule().assert_fix_explanation_contains(source, "line endings");
}

#[test]
fn converts_show_tabs_flag() {
    let source = "^cat -T file.txt";
    rule().assert_replacement_contains(source, "open --raw file.txt | lines");
    rule().assert_fix_explanation_contains(source, "-T");
    rule().assert_fix_explanation_contains(source, "tabs");
}

#[test]
fn converts_show_all_flag() {
    let source = "^cat -A file.txt";
    rule().assert_replacement_contains(source, "open --raw file.txt | lines");
    rule().assert_fix_explanation_contains(source, "-E");
}

#[test]
fn combines_number_with_multiple_files() {
    let source = "^cat -n file1.txt file2.txt";
    rule().assert_replacement_contains(source, "lines");
    rule().assert_replacement_contains(source, "enumerate");
}
