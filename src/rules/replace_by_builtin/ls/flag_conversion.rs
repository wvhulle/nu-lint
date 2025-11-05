use crate::rules::replace_by_builtin::ls::rule;

#[test]
fn converts_all_flag() {
    let source = "^ls -a";
    rule().assert_fix_contains(source, "ls --all");
    rule().assert_fix_description_contains(source, "--all");
}

#[test]
fn converts_combined_flags() {
    let source = "^ls -la";
    rule().assert_fix_contains(source, "ls --all");
    rule().assert_fix_description_contains(source, "-l");
    rule().assert_fix_description_contains(source, "not needed");
}

#[test]
fn converts_human_readable_flag() {
    let source = "^ls -h";
    rule().assert_fix_contains(source, "ls");
    rule().assert_fix_description_contains(source, "-h");
    rule().assert_fix_description_contains(source, "not needed");
}

#[test]
fn converts_long_flag() {
    let source = "^ls -l";
    rule().assert_fix_contains(source, "ls");
    rule().assert_fix_description_contains(source, "-l");
    rule().assert_fix_description_contains(source, "not needed");
}

#[test]
fn converts_long_format_all_flag() {
    let source = "^ls --all";
    rule().assert_fix_contains(source, "ls --all");
}

#[test]
fn mentions_recursive_alternative() {
    let source = "^ls -R";
    rule().assert_fix_description_contains(source, "recursive");
    rule().assert_fix_description_contains(source, "glob");
}
