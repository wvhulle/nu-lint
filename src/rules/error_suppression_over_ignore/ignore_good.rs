use super::rule;

#[test]
fn test_external_command_ignore_acceptable() {
    let acceptable_code = r"
^bluetoothctl power on | ignore
";
    rule().assert_ignores(acceptable_code);
}

#[test]
fn test_do_i_for_error_suppression() {
    let good_code = r"
do -i { rm /tmp/file.txt }
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_non_file_operation_ignore_acceptable() {
    let acceptable_code = r"
echo 'hello world' | ignore
";
    rule().assert_ignores(acceptable_code);
}

#[test]
fn test_save_with_ignore_acceptable() {
    let acceptable_code = r"
$data | save output.json | ignore
";
    rule().assert_ignores(acceptable_code);
}

#[test]
fn test_complex_pipeline_without_file_ops_acceptable() {
    let acceptable_code = r"
some | pipeline | each { |x| process $x } | ignore
";
    rule().assert_ignores(acceptable_code);
}

#[test]
fn test_external_rm_acceptable() {
    let acceptable_code = r"
^rm -f /tmp/file.txt | ignore
";
    rule().assert_ignores(acceptable_code);
}

#[test]
fn test_try_catch_pattern() {
    let acceptable_code = r#"
try { rm /tmp/file.txt } catch { print "failed" }
"#;
    rule().assert_ignores(acceptable_code);
}

#[test]
fn test_http_get_with_ignore_acceptable() {
    let acceptable_code = r"
http get https://api.example.com/data | ignore
";
    rule().assert_ignores(acceptable_code);
}
