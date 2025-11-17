use super::rule;
use crate::log::instrument;

#[test]
fn test_fix_curl_with_from_json() {
    instrument();
    let bad_code = "^curl https://api.example.com | from json";

    rule().assert_replacement_contains(bad_code, "let result =");
    rule().assert_replacement_contains(bad_code, "| complete");
    rule().assert_replacement_contains(bad_code, "if $result.exit_code != 0");
    rule().assert_replacement_contains(bad_code, "error make");
    rule().assert_replacement_contains(bad_code, "$result.stdout");
}

#[test]
fn test_fix_grep_with_lines() {
    instrument();
    let bad_code = "^grep 'pattern' file.txt | lines";

    rule().assert_replacement_contains(bad_code, "let result =");
    rule().assert_replacement_contains(bad_code, "^grep 'pattern' file.txt | lines | complete");
    rule().assert_replacement_contains(bad_code, "if $result.exit_code != 0");
}

#[test]
fn test_fix_ssh_with_lines() {
    instrument();
    let bad_code = "^ssh server 'ls -la' | lines";

    rule().assert_replacement_contains(bad_code, "let result =");
    rule().assert_replacement_contains(bad_code, "| complete");
}

#[test]
fn test_fix_wget_with_processing() {
    instrument();
    let bad_code = "^wget -qO- https://example.com | lines | where $it != ''";

    rule().assert_replacement_contains(bad_code, "let result =");
    rule().assert_replacement_contains(bad_code, "| complete");
}

#[test]
fn test_fix_external_with_parse() {
    instrument();
    let bad_code = "^systemctl status | parse '{service} {status}'";

    rule().assert_replacement_contains(bad_code, "let result =");
    rule().assert_replacement_contains(bad_code, "| complete");
}
