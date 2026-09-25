use super::RULE;

#[test]
fn fix_simple_cat_to_open_raw() {
    RULE.assert_fixed_is("^cat file.txt", "open --raw file.txt");
}

#[test]
fn fix_multiple_files_to_each_open() {
    RULE.assert_fixed_is(
        "^cat file1.txt file2.txt",
        "[file1.txt file2.txt] | each {|f| open --raw $f} | str join",
    );
}

#[test]
fn fix_number_lines_flag() {
    RULE.assert_fixed_is(
        "^cat -n file.txt",
        "open --raw file.txt | lines | enumerate",
    );
}

#[test]
fn fix_number_nonblank_flag() {
    RULE.assert_fixed_is(
        "^cat -b file.txt",
        r#"open --raw file.txt | lines | enumerate | where $it.item != """#,
    );
}

#[test]
fn fix_display_only_flags_drop_to_lines() {
    RULE.assert_fixed_is("^cat -E file.txt", "open --raw file.txt | lines");
    RULE.assert_fixed_is("^cat --show-all file.txt", "open --raw file.txt | lines");
}

#[test]
fn fix_combines_number_with_multiple_files() {
    RULE.assert_fixed_is(
        "^cat -n file1.txt file2.txt",
        "open --raw file1.txt file2.txt | lines | enumerate",
    );
}

#[test]
fn fix_without_file_reads_pipeline_input() {
    RULE.assert_fixed_is("^cat -n", "lines | enumerate");
}

#[test]
fn fix_inside_alias_definition() {
    RULE.assert_fixed_is("alias c = cat notes.txt", "alias c = open --raw notes.txt");
}

#[test]
fn no_fix_for_plain_cat_without_file() {
    RULE.assert_detects_without_fix("^cat");
}
