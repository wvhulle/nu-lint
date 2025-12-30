use super::RULE;
use crate::log::instrument;

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
    RULE.assert_ignores(good_code);
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
    RULE.assert_ignores(good_code);
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
    RULE.assert_ignores(good_code);
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
    RULE.assert_ignores(good_code);
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
    RULE.assert_ignores(good_code);
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
    RULE.assert_ignores(good_code);
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
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_library_with_focused_functions() {
    instrument();
    let good_code = r#"
export def fetch-data [] {
    http get https://api.example.com/data
}

export def save-data [data] {
    $data | save output.json
}

export def log-message [msg] {
    print $msg
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_pure_library_functions() {
    instrument();
    let good_code = r#"
export def transform [data] {
    $data | each { |x| $x * 2 }
}

export def calculate [x: int, y: int] {
    $x + $y
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_top_level_single_io_type() {
    instrument();
    let good_code = r#"
http get https://api.example.com/data1
http get https://api.example.com/data2
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_top_level_file_operations() {
    instrument();
    let good_code = r"
ls | save files.txt
cp input.txt output.txt
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignores_top_level_pure_script() {
    instrument();
    let good_code = r"
let x = 42
let y = $x * 2
$x + $y
";
    RULE.assert_ignores(good_code);
}
