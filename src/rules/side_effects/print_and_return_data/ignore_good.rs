use super::RULE;
use crate::log::init_env_log;

#[test]
fn ignores_function_with_only_print() {
    init_env_log();
    let good_code = r#"
def log-message [msg: string] {
    print $msg
    print "Done"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_function_returning_nothing() {
    init_env_log();
    let good_code = r#"
def process-data [] {
    print "Processing"
    mkdir /tmp/output
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_side_effect_only_function() {
    init_env_log();
    let good_code = r#"
def setup [] {
    mkdir /tmp/dir
    touch /tmp/dir/file.txt
    print "Setup complete"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_separate_verbose_and_quiet_versions() {
    init_env_log();
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
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_function_with_type_annotation_nothing() {
    init_env_log();
    let good_code = r#"
def notify []: nothing -> nothing {
    print "Notification sent"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_main_function() {
    init_env_log();
    let good_code = r#"
def main [] {
    print "Starting"
    [1 2 3]
}
"#;
    RULE.assert_ignores(good_code);
}
