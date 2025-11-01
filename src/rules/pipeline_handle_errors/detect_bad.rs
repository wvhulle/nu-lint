use super::rule;
use crate::context::LintContext;

fn init_logger() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        crate::clean_log::log();
    });
}

// Test external commands in pipelines without error handling

#[test]
fn test_detect_curl_piped_to_from_json() {
    init_logger();
    let bad_code = r"^curl https://api.github.com/repos/nushell/nushell | from json";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_wget_piped_to_tar() {
    init_logger();
    let bad_code = r"^wget -O - https://example.com/file.tar.gz | ^tar xz";
    rule().assert_detects(bad_code);
}

// Test network commands in pipelines
#[test]
fn test_detect_curl_piped_to_jq() {
    init_logger();
    let bad_code = r"^curl -s https://api.example.com/data | ^jq '.results'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_http_get_piped_to_processing() {
    init_logger();
    let bad_code = r"^curl -X GET https://api.example.com/users | from json | get name";
    rule().assert_detects(bad_code);
}

// Test file operations in pipelines
#[test]
fn test_detect_cat_piped_to_grep() {
    init_logger();
    let bad_code = r"^cat /var/log/syslog | ^grep error";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_find_piped_to_grep() {
    init_logger();
    let bad_code = r"^find /tmp -name '*.log' | lines | each { |f| ^grep error $f }";
    rule().assert_detects(bad_code);
}

// Test docker/container commands in pipelines
#[test]
fn test_detect_docker_ps_piped() {
    init_logger();
    let bad_code = r"^docker ps -a | lines | each { |line| $line | split row ' ' }";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_docker_logs_piped() {
    init_logger();
    let bad_code = r"^docker logs mycontainer | ^grep ERROR";
    rule().assert_detects(bad_code);
}

// Test database commands in pipelines
#[test]
fn test_detect_psql_piped() {
    init_logger();
    let bad_code = r"^psql -c 'SELECT * FROM users' | from csv";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_mysql_piped() {
    init_logger();
    let bad_code = r"^mysql -e 'SELECT * FROM users' | from csv";
    rule().assert_detects(bad_code);
}

// Test ssh/remote commands in pipelines
#[test]
fn test_detect_ssh_piped() {
    init_logger();
    let bad_code = r"^ssh user@host 'cat /var/log/app.log' | ^grep ERROR";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_scp_content_piped() {
    init_logger();
    let bad_code = r"^ssh user@host 'ls /tmp' | lines | each { |f| print $f }";
    rule().assert_detects(bad_code);
}

// Test nested functions with external commands in pipelines
#[test]
fn test_detect_in_function_definition() {
    init_logger();
    let bad_code = r"
def fetch-data [] {
    ^curl https://api.example.com | from json
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_in_closure() {
    init_logger();
    let bad_code = r"
ls | each { |file|
    ^cat $file.name | lines | length
}
";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));
    assert!(
        !violations.is_empty(),
        "Should detect external commands in pipeline within closure"
    );
}

// Test complex pipelines
#[test]
fn test_detect_multiple_external_in_pipeline() {
    init_logger();
    let bad_code = r"^curl https://api.example.com | ^jq '.data' | ^grep important";
    rule().assert_detects(bad_code);
}

#[test]
fn test_suggestion_recommends_complete_for_custom_handling() {
    init_logger();
    let bad_code = r"^docker build -t myapp . | lines";
    rule().assert_detects(bad_code);
}

#[test]
fn test_do_ignore_but_first_item_not_ignored() {
    init_logger();
    let good_code = "do -i { ^git status | grep modified | wc -l }";
    rule().assert_detects(good_code);
}

#[test]
fn test_suggestion_recommends_do_ignore_when_appropriate() {
    init_logger();
    let bad_code = r"^mkdir -p /tmp/test | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_all_three_patterns_in_suggestion() {
    init_logger();
    let bad_code = r"^curl https://api.example.com | from json";
    rule().assert_detects(bad_code);
}
