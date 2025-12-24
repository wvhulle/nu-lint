use super::RULE;
use crate::log::instrument;

#[test]
fn detects_file_and_network_io() {
    instrument();
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
    instrument();
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
    instrument();
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
    instrument();
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
    instrument();
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
    instrument();
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
    instrument();
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
    instrument();
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
