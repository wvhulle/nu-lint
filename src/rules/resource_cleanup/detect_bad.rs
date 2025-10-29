use super::rule;

#[test]
fn detect_file_open_without_collection() {
    rule().assert_detects("open large_file.txt | lines | where { $in | str contains 'error' }");
}

#[test]
fn detect_network_request_without_complete() {
    rule().assert_detects("http get api.example.com/data | get items | length");
}

#[test]
fn detect_external_process_without_management() {
    rule().assert_detects("^git log --oneline | lines | take 10");
}

#[test]
fn detect_file_processing_chain() {
    rule().assert_detects("open config.json | get database | get host");
}

#[test]
fn detect_curl_without_complete() {
    rule().assert_detects("^curl -s https://api.github.com/users/octocat | from json");
}

#[test]
fn detect_file_stream_processing() {
    rule().assert_detects("open log.txt | lines | where { $in | str contains 'ERROR' } | length");
}

#[test]
fn detect_multiple_file_operations() {
    rule().assert_detects("ls *.json | get name | each { open $in | get version }");
}