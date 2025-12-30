use super::RULE;
use crate::log::instrument;

#[test]
fn fix_simple_eza_to_ls() {
    instrument();
    let source = r"^eza";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls");
}

#[test]
fn fix_eza_all_to_ls_a() {
    instrument();
    let source = r"^eza -a";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls -a");
}

#[test]
fn fix_eza_long_all_to_ls_a() {
    instrument();
    let source = r"^eza --all";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls -a");
}

#[test]
fn fix_eza_long_view_to_ls_l() {
    instrument();
    let source = r"^eza -l";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls -l");
}

#[test]
fn fix_eza_combined_flags() {
    instrument();
    let source = r"^eza -la";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls -a -l");
}

#[test]
fn fix_eza_recurse_to_glob() {
    instrument();
    let source = r"^eza -R";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls **/*");
}

#[test]
fn fix_eza_tree_to_glob() {
    instrument();
    let source = r"^eza -T";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls **/*");
}

#[test]
fn fix_eza_sort_size() {
    instrument();
    let source = r"^eza --sort=size";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "sort-by size");
}

#[test]
fn fix_eza_sort_modified() {
    instrument();
    let source = r"^eza -s modified";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "sort-by modified");
}

#[test]
fn fix_eza_reverse() {
    instrument();
    let source = r"^eza -r --sort=size";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "--reverse");
}

#[test]
fn fix_eza_only_dirs() {
    instrument();
    let source = r"^eza -D";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "where type == dir");
}

#[test]
fn fix_eza_only_files() {
    instrument();
    let source = r"^eza -f";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "where type == file");
}

#[test]
fn fix_eza_with_path() {
    instrument();
    let source = r"^eza src";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls src");
}

#[test]
fn fix_eza_recurse_with_path() {
    instrument();
    let source = r"^eza -R src";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "ls src/**/*");
}

#[test]
fn fix_explanation_mentions_structured_data() {
    instrument();
    let source = r"^eza";
    RULE.assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_explanation_mentions_hidden_files_for_all() {
    instrument();
    let source = r"^eza -a";
    RULE.assert_fix_explanation_contains(source, "Hidden files");
}

#[test]
fn fix_explanation_mentions_long_view() {
    instrument();
    let source = r"^eza -l";
    RULE.assert_fix_explanation_contains(source, "Long view");
}

#[test]
fn fix_explanation_mentions_recursion_for_tree() {
    instrument();
    let source = r"^eza -T";
    RULE.assert_fix_explanation_contains(source, "Recursion");
}

#[test]
fn fix_explanation_mentions_dirs_only() {
    instrument();
    let source = r"^eza -D";
    RULE.assert_fix_explanation_contains(source, "Directories only");
}

#[test]
fn fix_explanation_mentions_sorting() {
    instrument();
    let source = r"^eza --sort=size";
    RULE.assert_fix_explanation_contains(source, "Sorting");
}

#[test]
fn fix_explanation_mentions_benefits() {
    instrument();
    let source = r"^eza";
    RULE.assert_fix_explanation_contains(source, "Benefits");
}

#[test]
fn fix_explanation_mentions_where_command() {
    instrument();
    let source = r"^eza";
    RULE.assert_fix_explanation_contains(source, "where");
}

#[test]
fn fix_explanation_mentions_sort_by() {
    instrument();
    let source = r"^eza";
    RULE.assert_fix_explanation_contains(source, "sort-by");
}
