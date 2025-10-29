use super::rule;

#[test]
fn detect_rm_rf_root() {
    rule().assert_detects("rm -rf /");
}

#[test]
fn detect_rm_rf_wildcard() {
    rule().assert_detects("rm -rf /*");
}

#[test]
fn detect_rm_with_unchecked_variable() {
    rule().assert_detects("let path = $env.HOME; rm -rf $path");
}

#[test]
fn detect_dangerous_parent_directory() {
    rule().assert_detects("rm -rf ../");
}

#[test]
fn detect_external_rm_dangerous() {
    rule().assert_detects("^rm -rf /tmp/*");
}

#[test]
fn detect_mv_to_dangerous_path() {
    rule().assert_detects("mv important_file /");
}

#[test]
fn detect_rm_variable_without_validation() {
    let bad_code = r"
def cleanup [path] {
    rm -rf $path
}";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_recursive_delete_in_pipeline() {
    rule().assert_detects("ls | where type == dir | get name | each { rm -rf $in }");
}
