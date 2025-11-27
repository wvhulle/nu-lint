use crate::rules::posix::ls::rule;

#[test]
fn fix_converts_external_ls_to_builtin() {
    let source = "^ls";
    rule().assert_replacement_contains(source, "ls");
    rule().assert_fix_explanation_contains(source, "structured");
    rule().assert_fix_explanation_contains(source, "data");
}

#[test]
fn fix_converts_ls_with_directory_path() {
    let source = "^ls /tmp";
    rule().assert_replacement_contains(source, "ls /tmp");
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_converts_ls_with_multiple_paths() {
    let source = "^ls src tests";
    rule().assert_replacement_contains(source, "ls src tests");
}

#[test]
fn fix_converts_ls_with_glob_pattern() {
    let source = "^ls *.rs";
    rule().assert_replacement_contains(source, "ls *.rs");
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_converts_ls_all_flag_to_builtin_all() {
    let source = "^ls -a";
    rule().assert_replacement_contains(source, "ls --all");
    rule().assert_fix_explanation_contains(source, "--all");
}

#[test]
fn fix_strips_unnecessary_flags_from_combined() {
    let source = "^ls -la";
    rule().assert_replacement_contains(source, "ls --all");
    rule().assert_fix_explanation_contains(source, "-l");
    rule().assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn fix_explains_human_readable_not_needed() {
    let source = "^ls -h";
    rule().assert_replacement_contains(source, "ls");
    rule().assert_fix_explanation_contains(source, "-h");
    rule().assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn fix_explains_long_flag_not_needed() {
    let source = "^ls -l";
    rule().assert_replacement_contains(source, "ls");
    rule().assert_fix_explanation_contains(source, "-l");
    rule().assert_fix_explanation_contains(source, "not needed");
}

#[test]
fn fix_converts_long_format_all_flag() {
    let source = "^ls --all";
    rule().assert_replacement_contains(source, "ls --all");
}

#[test]
fn fix_mentions_recursive_alternative() {
    let source = "^ls -R";
    rule().assert_fix_explanation_contains(source, "recursive");
    rule().assert_fix_explanation_contains(source, "glob");
}

#[test]
fn fix_converts_sort_by_time() {
    let source = "^ls -t";
    rule().assert_replacement_contains(source, "ls | sort-by modified");
    rule().assert_fix_explanation_contains(source, "sort-by modified");
}

#[test]
fn fix_converts_sort_by_size() {
    let source = "^ls -S";
    rule().assert_replacement_contains(source, "ls | sort-by size");
    rule().assert_fix_explanation_contains(source, "sort-by size");
}

#[test]
fn fix_converts_reverse_sort() {
    let source = "^ls -r";
    rule().assert_replacement_contains(source, "ls | reverse");
    rule().assert_fix_explanation_contains(source, "reverse");
}

#[test]
fn fix_combines_sort_and_reverse() {
    let source = "^ls -tr";
    rule().assert_replacement_contains(source, "ls | sort-by modified | reverse");
    rule().assert_fix_explanation_contains(source, "sort-by modified");
    rule().assert_fix_explanation_contains(source, "reverse");
}

#[test]
fn fix_combines_all_sort_with_reverse() {
    let source = "^ls -Str";
    rule().assert_replacement_contains(source, "ls | sort-by modified | sort-by size | reverse");
}

#[test]
fn fix_combines_flags_with_path() {
    let source = "^ls -lat /var/log";
    rule().assert_replacement_contains(source, "ls /var/log --all | sort-by modified");
    rule().assert_fix_explanation_contains(source, "-l");
    rule().assert_fix_explanation_contains(source, "not needed");
}
