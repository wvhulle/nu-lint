use crate::rules::replace_by_builtin::ls::rule;

#[test]
fn converts_ls_all_flag_to_builtin_all() {
    let source = "^ls -a";
    rule().assert_fix_contains(source, "ls --all");
    rule().assert_fix_explanation_contains(source, "--all");
}

#[test]
fn converts_ls_combined_flags_strips_unnecessary() {
    let source = "^ls -la";
    rule().assert_fix_contains(source, "ls --all");
    rule().assert_fix_explanation_contains(source, "-l");
    rule().assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn converts_ls_human_readable_flag_not_needed() {
    let source = "^ls -h";
    rule().assert_fix_contains(source, "ls");
    rule().assert_fix_explanation_contains(source, "-h");
    rule().assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn converts_ls_long_flag_not_needed() {
    let source = "^ls -l";
    rule().assert_fix_contains(source, "ls");
    rule().assert_fix_explanation_contains(source, "-l");
    rule().assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn converts_long_format_all_flag() {
    let source = "^ls --all";
    rule().assert_fix_contains(source, "ls --all");
}

#[test]
fn mentions_recursive_alternative() {
    let source = "^ls -R";
    rule().assert_fix_explanation_contains(source, "recursive");
    rule().assert_fix_explanation_contains(source, "glob");
}
