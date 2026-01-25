use super::RULE;
use crate::log::init_test_log;

#[test]
fn detects_file_and_network_io() {
    init_test_log();
    let bad_code = r"
def main [] {}

def sync-data [] {
    http get https://api.example.com/data | save output.json
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_file_and_print() {
    init_test_log();
    let bad_code = r#"
def main [] {}

def save-with-log [data] {
    print "Saving data..."
    $data | save output.json
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_network_and_print() {
    init_test_log();
    let bad_code = r#"
def main [] {}

def fetch-with-log [] {
    print "Fetching..."
    http get https://api.example.com/data
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_all_three_io_types() {
    init_test_log();
    let bad_code = r#"
def main [] {}

def process-all [] {
    print "Processing..."
    let data = (http get https://api.example.com/data)
    $data | save output.json
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_file_operations_mixed() {
    init_test_log();
    let bad_code = r#"
def main [] {}

def backup-and-notify [] {
    mkdir /tmp/backup
    print "Backup created"
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_nested_mixed_io() {
    init_test_log();
    let bad_code = r#"
def main [] {}

def complex-operation [] {
    if true {
        print "Starting..."
        http get https://api.example.com/data | save result.json
    }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_multiple_file_ops_with_network() {
    init_test_log();
    let bad_code = r"
def main [] {}

def download-and-archive [] {
    http get https://api.example.com/file | save /tmp/download.txt
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_print_with_multiple_file_ops() {
    init_test_log();
    let bad_code = r#"
def main [] {}

def setup-directories [] {
    print "Creating directories..."
    mkdir /tmp/dir1
    mkdir /tmp/dir2
    touch /tmp/dir1/file.txt
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_exported_functions_with_mixed_io() {
    init_test_log();
    let bad_code = r#"
def main [] {}

export def sync [] {
    print "Syncing..."
    http get https://api.example.com/data | save output.json
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_function_without_main() {
    init_test_log();
    let bad_code = r#"
def helper [] {
    print "Helper"
    http get https://api.example.com/data | save output.json
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_multiple_functions_without_main() {
    init_test_log();
    let bad_code = r#"
def fetch-data [] {
    print "Fetching..."
    http get https://api.example.com/data
}

def save-result [data] {
    print "Saving..."
    $data | save output.json
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_library_function_mixing_io() {
    init_test_log();
    let bad_code = r#"
export def sync-data [] {
    print "Syncing data..."
    let data = (http get https://api.example.com/data)
    $data | save local-cache.json
    print "Done"
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_top_level_script_mixing_io() {
    init_test_log();
    let bad_code = r#"
print "Fetching data..."
let data = (http get https://api.example.com/data)
$data | save output.json
print "Done"
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_top_level_script_network_and_file() {
    init_test_log();
    let bad_code = r"
http get https://api.example.com/file | save download.txt
";
    RULE.assert_detects(bad_code);
}
