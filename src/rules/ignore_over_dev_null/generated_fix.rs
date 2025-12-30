use super::RULE;

#[test]
fn fix_stderr_redirect() {
    let source = r"^evtest $keyboard err> /dev/null | lines";
    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "^evtest $keyboard e>| ignore | lines");
}

#[test]
fn fix_curl_redirect() {
    let source = r"^curl https://example.com err> /dev/null | from json";
    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "^curl https://example.com e>| ignore | from json");
}

#[test]
fn fix_grep_redirect() {
    let source = r"^grep 'pattern' file.txt err> /dev/null | lines";
    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "^grep 'pattern' file.txt e>| ignore | lines");
}

#[test]
fn fix_stdout_redirect() {
    let source = r"^noisy-cmd out> /dev/null | complete | get exit_code";
    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "^noisy-cmd o>| ignore | complete | get exit_code");
}

#[test]
fn fix_both_streams_redirect() {
    let source = r"^cmd out> /dev/null err> /dev/null | lines";
    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "^cmd o+e>| ignore | lines");
}

#[test]
fn multiple_fixes() {
    let source = r"
^curl https://api.example.com err> /dev/null | from json
^grep 'pattern' file.txt err> /dev/null | lines
";
    RULE.assert_count(source, 2);
    RULE.assert_fixed_contains(source, "e>| ignore");
}
