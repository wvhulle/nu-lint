use super::rule;
use crate::log::instrument;

#[test]
fn ignores_function_with_only_print() {
    instrument();
    let good_code = r#"
def log-message [msg: string] {
    print $msg
    print "Done"
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_function_with_only_return() {
    instrument();
    let good_code = r"
def get-data [] {
    http get https://api.example.com/data
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_function_with_print_to_stderr() {
    instrument();
    let good_code = r#"
def fetch-data [] {
    print -e "Fetching..."
    http get https://api.example.com/data
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_function_returning_nothing() {
    instrument();
    let good_code = r#"
def process-data [] {
    print "Processing"
    mkdir /tmp/output
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_pure_function() {
    instrument();
    let good_code = r"
def add [a: int, b: int] {
    $a + $b
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_side_effect_only_function() {
    instrument();
    let good_code = r#"
def setup [] {
    mkdir /tmp/dir
    touch /tmp/dir/file.txt
    print "Setup complete"
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_function_with_no_output() {
    instrument();
    let good_code = r#"
def save-data [data] {
    print "Saving data"
    $data | save output.json
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_separate_verbose_and_quiet_versions() {
    instrument();
    let good_code = r#"
# The quiet version - no print, just data
def get-data [] {
    http get https://api.example.com/data
}

# A separate verbose function that only prints, doesn't return data
def show-data-status [] {
    print "Fetching data..."
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_function_with_type_annotation_nothing() {
    instrument();
    let good_code = r#"
def notify []: nothing -> nothing {
    print "Notification sent"
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_main_function() {
    instrument();
    let good_code = r#"
def main [] {
    print "Starting"
    [1 2 3]
}
"#;
    rule().assert_ignores(good_code);
}
