use super::RULE;

#[test]
fn detects_external_ls() {
    let source = "^ls";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_ls_with_directory_path() {
    let source = "^ls /tmp";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_ls_with_multiple_paths() {
    let source = "^ls src tests";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_ls_with_glob_pattern() {
    let source = "^ls *.rs";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_all_flag() {
    let source = "^ls -a";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_combined_flags() {
    let source = "^ls -la";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_human_readable_flag() {
    let source = "^ls -h";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_long_flag() {
    let source = "^ls -l";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_long_format_all_flag() {
    let source = "^ls --all";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_recursive_flag() {
    let source = "^ls -R";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_sort_by_time() {
    let source = "^ls -t";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_sort_by_size() {
    let source = "^ls -S";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_reverse_sort() {
    let source = "^ls -r";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_combined_sort_and_reverse() {
    let source = "^ls -tr";
    RULE.assert_detects(source);
}

#[test]
fn detects_ls_with_flags_and_path() {
    let source = "^ls -lat /var/log";
    RULE.assert_detects(source);
}
