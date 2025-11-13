use super::rule;

#[test]
fn mixed_computation_with_print() {
    rule().assert_detects(
        r"
def process_data [] {
    let x = 10
    let y = 20
    let result = $x + $y
    print $result
}

def main [] {
    process_data
}
",
    );
}

#[test]
fn multiple_pure_statements_before_save() {
    rule().assert_detects(
        r"
def calculate_and_save [] {
    let numbers = 1..10
    let sum = ($numbers | math sum)
    let doubled = $sum * 2
    $doubled | save output.txt
}

def main [] {
    calculate_and_save
}
",
    );
}

#[test]
fn string_processing_with_print() {
    rule().assert_detects(
        r"
def format_message [] {
    let name = 'Alice'
    let greeting = $'Hello, ($name)!'
    let uppercase = ($greeting | str upcase)
    print $uppercase
}

def main [] {
    format_message
}
",
    );
}

#[test]
fn list_transformation_with_file_operation() {
    rule().assert_detects(
        r"
def process_list [] {
    let items = [1 2 3 4 5]
    let filtered = ($items | where $it > 2)
    let mapped = ($filtered | each { |x| $x * 2 })
    $mapped | save results.json
}

def main [] {
    process_list
}
",
    );
}

#[test]
fn mathematical_computation_with_exit() {
    rule().assert_detects(
        r"
def calculate_and_exit [] {
    let a = 5
    let b = 10
    let result = $a * $b
    if $result > 40 {
        exit 1
    }
}

def main [] {
    calculate_and_exit
}
",
    );
}

#[test]
fn data_preparation_with_mkdir() {
    rule().assert_detects(
        r"
def setup_environment [] {
    let base_path = '/tmp/project'
    let config_path = $base_path + '/config'
    mkdir $config_path
}

def main [] {
    setup_environment
}
",
    );
}

#[test]
fn computation_with_external_command() {
    rule().assert_detects(
        r"
def process_and_display [] {
    let data = [1 2 3]
    let processed = ($data | math sum)
    ^echo $processed
}

def main [] {
    process_and_display
}
",
    );
}

#[test]
fn multiple_computations_with_error_make() {
    rule().assert_detects(
        r"
def validate_and_error [] {
    let value = 42
    let threshold = 100
    let is_valid = $value < $threshold
    if not $is_valid {
        error make {msg: 'Value too high'}
    }
}

def main [] {
    validate_and_error
}
",
    );
}

#[test]
fn record_building_with_cd() {
    rule().assert_detects(
        r"
def prepare_and_navigate [] {
    let base = '/home/user'
    let project = 'my_project'
    let full_path = $base + '/' + $project
    cd $full_path
}

def main [] {
    prepare_and_navigate
}
",
    );
}

#[test]
fn complex_calculation_then_multiple_prints() {
    rule().assert_detects(
        r"
def calculate_and_report [] {
    let numbers = 1..100
    let sum = ($numbers | math sum)
    let avg = $sum / 100
    print $'Sum: {$sum}'
    print $'Average: {$avg}'
}

def main [] {
    calculate_and_report
}
",
    );
}

#[test]
fn data_transformation_then_http_post() {
    rule().assert_detects(
        r"
def send_data [] {
    let payload = {name: 'test', value: 42}
    let json = ($payload | to json)
    http post 'https://api.example.com' $json
}

def main [] {
    send_data
}
",
    );
}

#[test]
fn path_construction_then_rm() {
    rule().assert_detects(
        r"
def cleanup_temp [] {
    let temp_dir = '/tmp'
    let project_name = 'myproject'
    let full_path = $temp_dir + '/' + $project_name
    rm -rf $full_path
}

def main [] {
    cleanup_temp
}
",
    );
}

#[test]
fn filter_map_then_save() {
    rule().assert_detects(
        r"
def export_filtered_data [] {
    let raw_data = [{name: 'a', val: 1}, {name: 'b', val: 2}]
    let filtered = ($raw_data | where val > 1)
    let names = ($filtered | get name)
    $names | save output.txt
}

def main [] {
    export_filtered_data
}
",
    );
}

#[test]
fn config_building_then_save() {
    rule().assert_detects(
        r"
def create_config [] {
    let db_host = 'localhost'
    let db_port = 5432
    let config = {host: $db_host, port: $db_port}
    $config | save config.json
}

def main [] {
    create_config
}
",
    );
}

#[test]
fn string_manipulation_then_input() {
    rule().assert_detects(
        r"
def get_user_response [] {
    let prompt_prefix = '> '
    let question = 'Enter your name'
    let full_prompt = $prompt_prefix + $question
    input $full_prompt
}

def main [] {
    get_user_response
}
",
    );
}

#[test]
fn arithmetic_then_touch() {
    rule().assert_detects(
        r"
def create_numbered_file [] {
    let base = 'file'
    let num = 42
    let filename = $base + ($num | into string)
    touch $filename
}

def main [] {
    create_numbered_file
}
",
    );
}

#[test]
fn list_building_then_mv() {
    rule().assert_detects(
        r"
def move_file [] {
    let source_dir = '/tmp'
    let target_dir = '/home/user'
    let filename = 'data.txt'
    let source_path = $source_dir + '/' + $filename
    let target_path = $target_dir + '/' + $filename
    mv $source_path $target_path
}

def main [] {
    move_file
}
",
    );
}

#[test]
fn validation_computation_then_cp() {
    rule().assert_detects(
        r"
def backup_if_large [] {
    let threshold = 1000
    let size = 2000
    let should_backup = $size > $threshold
    if $should_backup {
        cp large_file.txt backup/
    }
}

def main [] {
    backup_if_large
}
",
    );
}

#[test]
fn multiple_string_ops_then_print() {
    rule().assert_detects(
        r"
def format_output [] {
    let first = 'Hello'
    let second = 'World'
    let combined = $first + ' ' + $second
    let final = ($combined | str upcase)
    print $final
}

def main [] {
    format_output
}
",
    );
}
