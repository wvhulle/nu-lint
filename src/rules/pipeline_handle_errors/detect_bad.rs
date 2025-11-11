use super::rule;
use crate::log::instrument;

#[test]
fn test_detect_curl_piped_to_from_json() {
    instrument();
    let bad_code = r"^curl https://api.github.com/repos/nushell/nushell | from json";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_wget_piped_to_tar() {
    instrument();
    let bad_code = r"^wget -O - https://example.com/file.tar.gz | ^tar xz";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_curl_piped_to_jq() {
    instrument();
    let bad_code = r"^curl -s https://api.example.com/data | ^jq '.results'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_http_get_piped_to_processing() {
    instrument();
    let bad_code = r"^curl -X GET https://api.example.com/users | from json | get name";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_cat_piped_to_grep() {
    instrument();
    let bad_code = r"^cat /var/log/syslog | ^grep error";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_find_piped_to_grep() {
    instrument();
    let bad_code = r"^find /tmp -name '*.log' | lines | each { |f| ^grep error $f }";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_docker_ps_piped() {
    instrument();
    let bad_code = r"^docker ps -a | lines | each { |line| $line | split row ' ' }";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_docker_logs_piped() {
    instrument();
    let bad_code = r"^docker logs mycontainer | ^grep ERROR";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_psql_piped() {
    instrument();
    let bad_code = r"^psql -c 'SELECT * FROM users' | from csv";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_mysql_piped() {
    instrument();
    let bad_code = r"^mysql -e 'SELECT * FROM users' | from csv";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_ssh_piped() {
    instrument();
    let bad_code = r"^ssh user@host 'cat /var/log/app.log' | ^grep ERROR";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_scp_content_piped() {
    instrument();
    let bad_code = r"^ssh user@host 'ls /tmp' | lines | each { |f| print $f }";
    rule().assert_detects(bad_code);
}
#[test]
fn test_detect_in_function_definition() {
    instrument();
    let bad_code = r"
def fetch-data [] {
    ^curl https://api.example.com | from json
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_in_closure() {
    instrument();
    let bad_code = r"
ls | each { |file|
    ^cat $file.name | lines | length
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_suggestion_recommends_complete_for_custom_handling() {
    instrument();
    let bad_code = r"^docker build -t myapp . | lines";
    rule().assert_detects(bad_code);
}

#[test]
fn test_first_item_not_ignored() {
    instrument();
    let bad_code = "do -i { ^git status | grep modified | wc -l }";
    rule().assert_detects(bad_code);
}
