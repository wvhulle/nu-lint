use crate::log::instrument;

use super::rule;

#[test]
fn ignores_only_file_operations() {
    instrument();
    let good_code = r"
def main [] {}

def save-files [data] {
    $data | save output.json
    cp output.json backup.json
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_only_network_operations() {
    instrument();
    let good_code = r"
def main [] {}

def fetch-multiple [] {
    let data1 = (http get https://api.example.com/data1)
    let data2 = (http get https://api.example.com/data2)
    [$data1, $data2]
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_only_print_operations() {
    instrument();
    let good_code = r#"
def main [] {}

def log-messages [] {
    print "Starting..."
    print "Processing..."
    print "Done"
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_pure_functions() {
    instrument();
    let good_code = r"
def main [] {}

def calculate [x: int, y: int] {
    $x + $y
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_main_function() {
    instrument();
    let good_code = r#"
def main [] {
    print "Starting..."
    http get https://api.example.com/data | save output.json
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_exported_functions() {
    instrument();
    let good_code = r#"
def main [] {}

export def sync [] {
    print "Syncing..."
    http get https://api.example.com/data | save output.json
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_function_without_main() {
    instrument();
    let good_code = r#"
def helper [] {
    print "Helper"
    http get https://api.example.com/data | save output.json
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_print_to_stderr_with_file_ops() {
    instrument();
    let good_code = r#"
def main [] {}

def save-with-debug [data] {
    print -e "Debug: saving..."
    $data | save output.json
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_single_io_type_multiple_times() {
    instrument();
    let good_code = r"
def main [] {}

def batch-save [data] {
    $data | save file1.json
    $data | save file2.json
    $data | save file3.json
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_no_io_operations() {
    instrument();
    let good_code = r"
def main [] {}

def transform [data] {
    $data | each { |x| $x * 2 } | where $it > 10
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_data_pipeline_operations() {
    instrument();
    let good_code = r"
def main [] {}

def process-list [items] {
    $items | where active | select name age | sort-by name
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_control_flow_only() {
    instrument();
    let good_code = r"
def main [] {}

def conditional-process [flag: bool] {
    if $flag {
        let x = 10
        $x * 2
    } else {
        0
    }
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignores_file_operations_in_closure() {
    instrument();
    let good_code = r"
def main [] {}

def batch-operations [files] {
    $files | each { |f| save $f }
}
";
    rule().assert_ignores(good_code);
}
