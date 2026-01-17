use super::RULE;

#[test]
fn fix_converts_external_ls_to_builtin() {
    let source = "^ls";
    RULE.assert_fixed_contains(source, "ls");
    RULE.assert_fix_explanation_contains(source, "structured");
    RULE.assert_fix_explanation_contains(source, "data");
}

#[test]
fn fix_converts_ls_with_directory_path() {
    let source = "^ls /tmp";
    RULE.assert_fixed_contains(source, "ls /tmp");
    RULE.assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_converts_ls_with_multiple_paths() {
    let source = "^ls src tests";
    RULE.assert_fixed_contains(source, "ls src tests");
}

#[test]
fn fix_converts_ls_with_glob_pattern() {
    let source = "^ls *.rs";
    RULE.assert_fixed_contains(source, "ls *.rs");
    RULE.assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_converts_ls_all_flag_to_builtin_all() {
    let source = "^ls -a";
    RULE.assert_fixed_contains(source, "ls --all");
    RULE.assert_fix_explanation_contains(source, "--all");
}

#[test]
fn fix_strips_unnecessary_flags_from_combined() {
    let source = "^ls -la";
    RULE.assert_fixed_contains(source, "ls --all");
    RULE.assert_fix_explanation_contains(source, "-l");
    RULE.assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn fix_explains_human_readable_not_needed() {
    let source = "^ls -h";
    RULE.assert_fixed_contains(source, "ls");
    RULE.assert_fix_explanation_contains(source, "-h");
    RULE.assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn fix_explains_long_flag_not_needed() {
    let source = "^ls -l";
    RULE.assert_fixed_contains(source, "ls");
    RULE.assert_fix_explanation_contains(source, "-l");
    RULE.assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn fix_converts_long_format_all_flag() {
    let source = "^ls --all";
    RULE.assert_fixed_contains(source, "ls --all");
}

#[test]
fn fix_mentions_recursive_alternative() {
    let source = "^ls -R";
    RULE.assert_fix_explanation_contains(source, "recursive");
    RULE.assert_fix_explanation_contains(source, "glob");
}

#[test]
fn fix_converts_sort_by_time() {
    let source = "^ls -t";
    RULE.assert_fixed_contains(source, "ls | sort-by modified");
    RULE.assert_fix_explanation_contains(source, "sort-by modified");
}

#[test]
fn fix_converts_sort_by_size() {
    let source = "^ls -S";
    RULE.assert_fixed_contains(source, "ls | sort-by size");
    RULE.assert_fix_explanation_contains(source, "sort-by size");
}

#[test]
fn fix_converts_reverse_sort() {
    let source = "^ls -r";
    RULE.assert_fixed_contains(source, "ls | reverse");
    RULE.assert_fix_explanation_contains(source, "reverse");
}

#[test]
fn fix_combines_sort_and_reverse() {
    let source = "^ls -tr";
    RULE.assert_fixed_contains(source, "ls | sort-by modified | reverse");
    RULE.assert_fix_explanation_contains(source, "sort-by modified");
    RULE.assert_fix_explanation_contains(source, "reverse");
}

#[test]
fn fix_combines_all_sort_with_reverse() {
    let source = "^ls -Str";
    RULE.assert_fixed_contains(source, "ls | sort-by modified | sort-by size | reverse");
}

#[test]
fn fix_combines_flags_with_path() {
    let source = "^ls -lat /var/log";
    RULE.assert_fixed_contains(source, "ls /var/log --all | sort-by modified");
    RULE.assert_fix_explanation_contains(source, "-l");
    RULE.assert_fix_explanation_contains(source, "not needed");
}
