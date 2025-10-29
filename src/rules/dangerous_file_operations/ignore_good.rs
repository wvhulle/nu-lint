use super::rule;

#[test]
fn ignore_safe_specific_file_removal() {
    rule().assert_ignores("rm temp.txt");
}

#[test]
fn ignore_rm_with_validation() {
    let good_code = r"
def cleanup [path] {
    if ($path | path exists) and ($path != '/') {
        rm -rf $path
    }
}";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_safe_directory_operations() {
    rule().assert_ignores("mkdir temp; cd temp; rm file.txt");
}

#[test]
fn ignore_rm_in_safe_directory() {
    rule().assert_ignores("cd /tmp/myproject; rm -rf build");
}

#[test]
fn ignore_validated_variable_usage() {
    let good_code = r"
let path = $env.TEMP_DIR
if ($path | str starts-with '/tmp/') {
    rm -rf $path
}";
    rule().assert_ignores(good_code);
}
