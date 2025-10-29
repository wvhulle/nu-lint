use super::rule;

#[test]
fn detect_long_pipeline_without_error_handling() {
    rule().assert_detects("ls | where type == file | get name | each { open $in } | to json");
}

#[test]
fn detect_external_command_pipeline() {
    rule().assert_detects("^git status | lines | where { $in | str contains 'modified' }");
}

#[test]
fn detect_complex_data_processing_pipeline() {
    rule().assert_detects("open data.json | get items | where active == true | get name | sort");
}

#[test]
fn detect_file_processing_pipeline() {
    rule().assert_detects("ls *.txt | get name | each { open $in | lines | length } | math sum");
}

#[test]
fn detect_external_tool_chain() {
    rule().assert_detects("^curl -s api.com/data | ^jq '.items[]' | lines");
}

#[test]
fn detect_network_operation_pipeline() {
    rule().assert_detects("http get api.example.com | get data | where status == 'active' | get id");
}

#[test]
fn detect_mixed_external_builtin_pipeline() {
    rule().assert_detects("^find . -name '*.nu' | lines | each { open $in } | where { $in | str contains 'def' }");
}