use super::RULE;

#[test]
fn test_detect_evtest_err_redirect() {
    let bad_code = r"^evtest $keyboard err> /dev/null | lines";
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_grep_err_redirect() {
    let bad_code = r"^grep 'pattern' file.txt err> /dev/null | lines";
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_curl_err_redirect() {
    let bad_code = r"^curl https://example.com err> /dev/null | from json";
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_find_err_redirect() {
    let bad_code = r"^find /path err> /dev/null | lines | where $it != ''";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_ssh_err_redirect() {
    let bad_code = r"^ssh server 'ls' err> /dev/null | lines";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_wget_err_redirect() {
    let bad_code = r"^wget -qO- https://example.com err> /dev/null | str trim";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_function() {
    let bad_code = r"
def fetch-data [] {
    ^curl https://api.example.com err> /dev/null | from json
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_with_variable() {
    let bad_code = r"let $url = 'https://example.com'; ^curl $url err> /dev/null | lines";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_multiple_redirects_in_file() {
    let bad_code = r"
^grep 'foo' file1.txt err> /dev/null | lines
^grep 'bar' file2.txt err> /dev/null | lines
";
    RULE.assert_count(bad_code, 2);
}
