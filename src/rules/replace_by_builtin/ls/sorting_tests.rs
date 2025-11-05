use crate::rules::replace_by_builtin::ls::rule;

#[test]
fn converts_sort_by_time() {
    let source = "^ls -t";
    rule().assert_fix_contains(source, "ls | sort-by modified");
    rule().assert_fix_description_contains(source, "sort-by modified");
}

#[test]
fn converts_sort_by_size() {
    let source = "^ls -S";
    rule().assert_fix_contains(source, "ls | sort-by size");
    rule().assert_fix_description_contains(source, "sort-by size");
}

#[test]
fn converts_reverse_sort() {
    let source = "^ls -r";
    rule().assert_fix_contains(source, "ls | reverse");
    rule().assert_fix_description_contains(source, "reverse");
}

#[test]
fn combines_sort_and_reverse() {
    let source = "^ls -tr";
    rule().assert_fix_contains(source, "ls | sort-by modified | reverse");
    rule().assert_fix_description_contains(source, "sort-by modified");
    rule().assert_fix_description_contains(source, "reverse");
}

#[test]
fn combines_all_sort_with_reverse() {
    let source = "^ls -Str";
    rule().assert_fix_contains(source, "ls | sort-by modified | sort-by size | reverse");
}

#[test]
fn combines_flags_with_path() {
    let source = "^ls -lat /var/log";
    rule().assert_fix_contains(source, "ls /var/log --all | sort-by modified");
    rule().assert_fix_description_contains(source, "-l");
    rule().assert_fix_description_contains(source, "not needed");
}
