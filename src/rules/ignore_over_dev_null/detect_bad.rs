use super::RULE;

#[test]
fn stderr_redirect() {
    RULE.assert_detects(r"^evtest $keyboard err> /dev/null | lines");
}

#[test]
fn grep_stderr_redirect() {
    RULE.assert_detects(r"^grep 'pattern' file.txt err> /dev/null | lines");
}

#[test]
fn curl_stderr_redirect() {
    RULE.assert_detects(r"^curl https://example.com err> /dev/null | from json");
}

#[test]
fn find_stderr_redirect() {
    RULE.assert_detects(r"^find /path err> /dev/null | lines | where $it != ''");
}

#[test]
fn in_function() {
    RULE.assert_detects(
        r"
def fetch-data [] {
    ^curl https://api.example.com err> /dev/null | from json
}
",
    );
}

#[test]
fn multiple_redirects() {
    RULE.assert_count(
        r"
^grep 'foo' file1.txt err> /dev/null | lines
^grep 'bar' file2.txt err> /dev/null | lines
",
        2,
    );
}
