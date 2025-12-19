use crate::log::instrument;

use super::rule;

#[test]
fn test_external_with_from_json() {
    let bad_code = r"^curl https://api.example.com | from json";
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_with_parse() {
    let bad_code = r#"^ps aux | lines | parse "{pid} {user}""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_with_where() {
    let bad_code = r"^curl https://example.com | lines | where $it =~ 'pattern'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_with_select() {
    let bad_code = r"^ps aux | from ssv | select user pid";
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_with_each() {
    let bad_code = r"^find . -name '*.rs' | lines | each { |f| $f | path basename }";
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_with_from_yaml() {
    let bad_code = r"^curl https://example.com/config.yaml | from yaml";
    rule().assert_detects(bad_code);
}

#[test]
fn test_curl_with_from_toml() {
    let bad_code = r"^curl https://api.example.com/config.toml | from toml";
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_with_group_by() {
    let bad_code = r"^ps aux | from ssv | group-by user";
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_nested_in_function() {
    let bad_code = r"
def fetch-data [] {
    ^curl https://api.example.com | from json
}
";
    rule().assert_detects(bad_code);
}

// Tests for simple processing (previously would not have been detected)
#[test]
fn test_external_with_lines() {
    let bad_code = r"^grep 'pattern' file.txt | lines";
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_with_str_trim() {
    let bad_code = r"^curl https://example.com | str trim";
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_with_split() {
    let bad_code = r"^cat file.txt | split row '\n'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_ssh_with_lines() {
    let bad_code = r"^ssh server 'ls -la' | lines";
    rule().assert_detects(bad_code);
}

#[test]
fn test_wget_with_simple_processing() {
    instrument();
    let bad_code = r"^wget -qO- https://example.com | lines | where $it != ''";
    rule().assert_detects(bad_code);
}
