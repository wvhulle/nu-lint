use crate::rules::posix_tools::ls::rule;

#[test]
fn detects_external_ls() {
    let source = "^ls";
    rule().assert_detects(source);
}

#[test]
fn detects_external_ls_with_directory_path() {
    let source = "^ls /tmp";
    rule().assert_detects(source);
}

#[test]
fn detects_external_ls_with_multiple_paths() {
    let source = "^ls src tests";
    rule().assert_detects(source);
}

#[test]
fn detects_external_ls_with_glob_pattern() {
    let source = "^ls *.rs";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_all_flag() {
    let source = "^ls -a";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_combined_flags() {
    let source = "^ls -la";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_human_readable_flag() {
    let source = "^ls -h";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_long_flag() {
    let source = "^ls -l";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_long_format_all_flag() {
    let source = "^ls --all";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_recursive_flag() {
    let source = "^ls -R";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_sort_by_time() {
    let source = "^ls -t";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_sort_by_size() {
    let source = "^ls -S";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_reverse_sort() {
    let source = "^ls -r";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_combined_sort_and_reverse() {
    let source = "^ls -tr";
    rule().assert_detects(source);
}

#[test]
fn detects_ls_with_flags_and_path() {
    let source = "^ls -lat /var/log";
    rule().assert_detects(source);
}
