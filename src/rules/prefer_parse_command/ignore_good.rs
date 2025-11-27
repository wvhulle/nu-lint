use super::rule;

#[test]
fn test_good_parse_command() {
    let good = "'name:john age:30' | parse '{name}:{age}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_with_patterns() {
    let good = "'User: alice, ID: 123' | parse 'User: {name}, ID: {id}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_simple_split() {
    let good = "'a,b,c' | split row ','";
    rule().assert_ignores(good);
}

#[test]
fn test_good_split_for_iteration() {
    let good = "'a,b,c' | split row ',' | each { |item| $item | str upcase }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_split_column() {
    let good = "'name,age,city' | split column ',' name age city";
    rule().assert_ignores(good);
}

#[test]
fn test_good_from_csv() {
    let good = "'name,age\njohn,30\njane,25' | from csv";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_user_data() {
    let good = r#"
let data = "user:john:1000"
let parsed = ($data | parse "{username}:{name}:{uid}")
let username = ($parsed | get username)
"#;
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_ip_port() {
    let good = "'192.168.1.100:8080' | parse '{ip}:{port}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_log_format() {
    let good = "'[2024-01-15] INFO: Server started' | parse '[{date}] {level}: {message}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_device_info() {
    let good = "'Device AA:BB:CC:DD:EE:FF MyDevice' | parse 'Device {mac} {name}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_email() {
    let good = "'user@example.com' | parse '{username}@{domain}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_key_value() {
    let good = "'key=value' | parse '{key}={value}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_with_regex() {
    let good = r"'foo bar baz' | parse --regex '\s+(?<word>\w+)\s+'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_split_without_indexing() {
    let good = r#"
let parts = ("a,b,c" | split row ",")
$parts | each { |p| print $p }
"#;
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_temperature() {
    let good = "'Temperature: 25.5°C' | parse 'Temperature: {temp}°C'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_structured_log() {
    let good = "'ERROR [module:function] Something went wrong' | parse '{level} \
                [{module}:{function}] {message}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_split_with_filter() {
    let good = r#"
"a,b,c,d" | split row "," | where $it != "b"
"#;
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_with_underscore_ignore() {
    let good = "'hello world' | parse '{word} {_}'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_from_json() {
    let good = r#"'{"name": "john", "age": 30}' | from json"#;
    rule().assert_ignores(good);
}

#[test]
fn test_good_parse_multiline() {
    let good = r#"
let logs = [
    "[INFO] Starting server"
    "[ERROR] Connection failed"
]
$logs | each { |line| $line | parse '[{level}] {message}' }
"#;
    rule().assert_ignores(good);
}

#[test]
fn test_ignore_parse_usage() {
    let good_code = r#"
open data.txt | parse "{name} {value}"
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_each_without_split() {
    let good_code = r"
seq 1 10 | each { |x| $x * 2 }
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_split_without_each() {
    let good_code = r#"
"one,two,three" | split row ","
"#;

    rule().assert_ignores(good_code);
}
